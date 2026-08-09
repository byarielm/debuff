# Proxmox LXC deployment (Tailscale Serve)

This deploys debuff privately on the neatopause LXC. Docker binds the service to loopback only; Tailscale Serve is the HTTPS ingress for devices on the tailnet.

This is the **private smoke-test** deployment. Do not do Bluesky OAuth or the labeler onboarding yet: the authorization server cannot fetch metadata from a tailnet-private URL. Enable Tailscale Funnel first (Phase 2) before onboarding.

## Prerequisites

- Docker Engine and the Docker Compose plugin installed in the LXC
- Tailscale connected in the LXC
- This branch, or your fork's equivalent, checked out in the LXC
- Set the LXC's canonical Tailscale DNS name in the untracked `deploy/.env`.

## First deployment

From the repository root **inside the LXC**:

```bash
sudo install -d -m 0700 /opt/neatopause/data /opt/neatopause/config
cd deploy
cp .env.example .env
chmod 600 .env
```

Generate and set the session secret in `deploy/.env`:

```bash
secret=$(openssl rand -hex 32)
sed -i "s/REPLACE_WITH_A_RANDOM_64_HEX_CHARACTER_SECRET/$secret/" .env
unset secret
```

Start the service:

```bash
docker compose --env-file .env -f compose.yaml up -d
docker compose --env-file .env -f compose.yaml ps
curl http://127.0.0.1:3000/health
```

Expected health response:

```json
{"status":"ok"}
```

## Tailscale Serve

Configure a persistent HTTPS reverse proxy on the LXC:

```bash
tailscale serve --bg --https=443 http://127.0.0.1:3000
tailscale serve status
```

From a device on the tailnet, verify:

```bash
curl https://neatopause.example-tailnet.ts.net/health
```

Then open the real `DEBUFF_PUBLIC_URL` from `deploy/.env` in a browser. The
frontend and API share the same origin, so no separate frontend container is
needed.

## Persistent state and backup

These host directories are stateful and must be backed up together:

```text
/opt/neatopause/data/     SQLite database and signing key
/opt/neatopause/config/   config.toml with the labeler DID/key path
```

Before an upgrade, snapshot or back up `/opt/neatopause`. To inspect logs:

```bash
cd /path/to/debuff/deploy
docker compose --env-file .env -f compose.yaml logs --follow
```

## Upgrade

Do not use `latest`. Change `DEBUFF_IMAGE` deliberately, back up state first, then run:

```bash
docker compose --env-file .env -f compose.yaml pull
docker compose --env-file .env -f compose.yaml up -d
```

## Next phase

Once private health/UI/WebSocket checks pass, enable **Tailscale Funnel** for the existing HTTPS Serve endpoint. Confirm that `/oauth/client-metadata.json` is publicly reachable before beginning Bluesky OAuth or labeler onboarding.
