INSERT INTO label_definitions (identifier, severity, blurs, default_setting, adult_only, builtin) VALUES
    ('!hide', 'alert', 'content', 'hide', 0, 1),
    ('!no-promote', 'none', 'none', 'hide', 0, 1),
    ('!warn', 'alert', 'content', 'warn', 0, 1),
    ('!no-unauthenticated', 'none', 'none', 'hide', 0, 1),
    ('dmca-violation', 'alert', 'content', 'hide', 0, 1),
    ('doxxing', 'alert', 'content', 'hide', 0, 1),
    ('porn', 'alert', 'media', 'hide', 1, 1),
    ('sexual', 'alert', 'media', 'warn', 1, 1),
    ('nudity', 'alert', 'media', 'warn', 1, 1),
    ('nsfl', 'alert', 'media', 'warn', 0, 1),
    ('gore', 'alert', 'media', 'warn', 0, 1);

INSERT INTO label_definition_locales (definition_id, lang, name, description) VALUES
    ((SELECT id FROM label_definitions WHERE identifier = '!hide'), 'en', 'Hide', 'Hides the content entirely'),
    ((SELECT id FROM label_definitions WHERE identifier = '!no-promote'), 'en', 'No Promote', 'Prevents content from appearing in recommendations'),
    ((SELECT id FROM label_definitions WHERE identifier = '!warn'), 'en', 'Warning', 'Shows a warning before displaying content'),
    ((SELECT id FROM label_definitions WHERE identifier = '!no-unauthenticated'), 'en', 'No Unauthenticated', 'Hides content from logged-out users'),
    ((SELECT id FROM label_definitions WHERE identifier = 'dmca-violation'), 'en', 'DMCA Violation', 'Content that violates copyright law'),
    ((SELECT id FROM label_definitions WHERE identifier = 'doxxing'), 'en', 'Doxxing', 'Content that reveals private personal information'),
    ((SELECT id FROM label_definitions WHERE identifier = 'porn'), 'en', 'Pornography', 'Explicit sexual content'),
    ((SELECT id FROM label_definitions WHERE identifier = 'sexual'), 'en', 'Sexual', 'Sexually suggestive content'),
    ((SELECT id FROM label_definitions WHERE identifier = 'nudity'), 'en', 'Nudity', 'Content containing nudity'),
    ((SELECT id FROM label_definitions WHERE identifier = 'nsfl'), 'en', 'NSFL', 'Content that is disturbing or not safe for life'),
    ((SELECT id FROM label_definitions WHERE identifier = 'gore'), 'en', 'Gore', 'Graphic violent content');
