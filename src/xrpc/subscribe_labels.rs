use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::AppState;
use crate::db::adapt_sql;

// ---------------------------------------------------------------------------
// Query parameters
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct SubscribeLabelsParams {
    pub cursor: Option<i64>,
}

// ---------------------------------------------------------------------------
// Database row type
// ---------------------------------------------------------------------------

struct LabelRow {
    seq: i64,
    ver: i32,
    src: String,
    uri: String,
    cid: Option<String>,
    val: String,
    neg: bool,
    cts: String,
    exp: Option<String>,
    sig: Vec<u8>,
}

// ---------------------------------------------------------------------------
// CBOR frame types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct DataFrameHeader {
    op: i32,
    t: &'static str,
}

#[derive(Serialize)]
struct ErrorFrameHeader {
    op: i32,
}

#[derive(Serialize)]
struct LabelFrameBody {
    seq: i64,
    labels: Vec<LabelCbor>,
}

#[derive(Serialize)]
struct LabelCbor {
    ver: i32,
    src: String,
    uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    cid: Option<String>,
    val: String,
    #[serde(skip_serializing_if = "is_false")]
    neg: bool,
    cts: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    exp: Option<String>,
    #[serde(with = "serde_bytes")]
    sig: Vec<u8>,
}

fn is_false(b: &bool) -> bool {
    !*b
}

#[derive(Serialize)]
struct ErrorFrameBody {
    error: String,
    message: String,
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

/// GET /xrpc/com.atproto.label.subscribeLabels?cursor=N
pub async fn subscribe_labels(
    State(state): State<AppState>,
    Query(params): Query<SubscribeLabelsParams>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, params.cursor))
}

async fn handle_socket(mut socket: WebSocket, state: AppState, cursor: Option<i64>) {
    // If a cursor was provided, replay historical labels first.
    if let Some(cursor) = cursor
        && send_historical(&mut socket, &state, cursor).await.is_err()
    {
        return;
    }

    // Subscribe to live label notifications.
    let mut rx = state.label_broadcast.subscribe();

    loop {
        match rx.recv().await {
            Ok(seq) => {
                if let Err(()) = send_label_by_seq(&mut socket, &state, seq).await {
                    return;
                }
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                warn!("subscribeLabels: broadcast lagged, missed {n} messages");
                continue;
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                info!("subscribeLabels: broadcast channel closed, shutting down");
                return;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Historical replay
// ---------------------------------------------------------------------------

async fn send_historical(socket: &mut WebSocket, state: &AppState, cursor: i64) -> Result<(), ()> {
    let backend = state.config.database.backend.clone();
    #[allow(clippy::type_complexity)]
    let rows: Vec<(
        i64,
        String,
        String,
        Option<String>,
        String,
        i32,
        String,
        Option<String>,
        Vec<u8>,
    )> = sqlx::query_as(&adapt_sql(
        "SELECT seq, src, uri, cid, val, neg, cts, exp, sig
             FROM labels
             WHERE seq > $1
             ORDER BY seq ASC",
        backend,
    ))
    .bind(cursor)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        warn!("subscribeLabels: failed to query historical labels: {e}");
    })?;

    // If cursor was provided but no rows found, check if cursor is too old.
    if rows.is_empty() {
        let min_seq: Option<(Option<i64>,)> = sqlx::query_as("SELECT MIN(seq) FROM labels")
            .fetch_optional(&state.db)
            .await
            .map_err(|e| {
                warn!("subscribeLabels: failed to check min seq: {e}");
            })?;

        if let Some((Some(min),)) = min_seq
            && cursor < min
        {
            let frame = encode_error_frame(
                "OutdatedCursor",
                &format!("cursor {cursor} is before the earliest available seq {min}"),
            );
            let _ = socket.send(Message::Binary(frame.into())).await;
            let _ = socket.send(Message::Close(None)).await;
            return Err(());
        }

        return Ok(());
    }

    for row in rows {
        let label = LabelRow {
            seq: row.0,
            ver: 1,
            src: row.1,
            uri: row.2,
            cid: row.3,
            val: row.4,
            neg: row.5 != 0,
            cts: row.6,
            exp: row.7,
            sig: row.8,
        };
        let frame = encode_label_frame(&label);
        if socket.send(Message::Binary(frame.into())).await.is_err() {
            return Err(());
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Send a single label by seq
// ---------------------------------------------------------------------------

async fn send_label_by_seq(socket: &mut WebSocket, state: &AppState, seq: i64) -> Result<(), ()> {
    let backend = state.config.database.backend.clone();
    #[allow(clippy::type_complexity)]
    let row: Option<(
        i64,
        String,
        String,
        Option<String>,
        String,
        i32,
        String,
        Option<String>,
        Vec<u8>,
    )> = sqlx::query_as(&adapt_sql(
        "SELECT seq, src, uri, cid, val, neg, cts, exp, sig
             FROM labels
             WHERE seq = $1",
        backend,
    ))
    .bind(seq)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        warn!("subscribeLabels: failed to query label seq={seq}: {e}");
    })?;

    let Some(row) = row else {
        warn!("subscribeLabels: label seq={seq} not found in database");
        return Ok(());
    };

    let label = LabelRow {
        seq: row.0,
        ver: 1,
        src: row.1,
        uri: row.2,
        cid: row.3,
        val: row.4,
        neg: row.5 != 0,
        cts: row.6,
        exp: row.7,
        sig: row.8,
    };

    let frame = encode_label_frame(&label);
    if socket.send(Message::Binary(frame.into())).await.is_err() {
        return Err(());
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Frame encoding (ATProto event stream format)
// ---------------------------------------------------------------------------

fn encode_label_frame(label: &LabelRow) -> Vec<u8> {
    let header = DataFrameHeader {
        op: 1,
        t: "#labels",
    };

    let label_cbor = LabelCbor {
        ver: label.ver,
        src: label.src.clone(),
        uri: label.uri.clone(),
        cid: label.cid.clone(),
        val: label.val.clone(),
        neg: label.neg,
        cts: label.cts.clone(),
        exp: label.exp.clone(),
        sig: label.sig.clone(),
    };

    let body = LabelFrameBody {
        seq: label.seq,
        labels: vec![label_cbor],
    };

    let mut buf = Vec::new();
    buf.extend_from_slice(&serde_ipld_dagcbor::to_vec(&header).expect("failed to encode header"));
    buf.extend_from_slice(&serde_ipld_dagcbor::to_vec(&body).expect("failed to encode body"));
    buf
}

fn encode_error_frame(error: &str, message: &str) -> Vec<u8> {
    let header = ErrorFrameHeader { op: -1 };

    let body = ErrorFrameBody {
        error: error.to_string(),
        message: message.to_string(),
    };

    let mut buf = Vec::new();
    buf.extend_from_slice(&serde_ipld_dagcbor::to_vec(&header).expect("failed to encode header"));
    buf.extend_from_slice(&serde_ipld_dagcbor::to_vec(&body).expect("failed to encode body"));
    buf
}
