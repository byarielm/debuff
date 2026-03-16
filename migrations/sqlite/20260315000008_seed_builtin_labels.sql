INSERT INTO label_definitions (identifier, severity, blurs, default_setting, adult_only, builtin) VALUES
    ('!hide', 'alert', 'content', 'hide', false, true),
    ('!no-promote', 'none', 'none', 'hide', false, true),
    ('!warn', 'alert', 'content', 'warn', false, true),
    ('!no-unauthenticated', 'none', 'none', 'hide', false, true),
    ('dmca-violation', 'alert', 'content', 'hide', false, true),
    ('doxxing', 'alert', 'content', 'hide', false, true),
    ('porn', 'alert', 'media', 'hide', true, true),
    ('sexual', 'alert', 'media', 'warn', true, true),
    ('nudity', 'alert', 'media', 'warn', true, true),
    ('nsfl', 'alert', 'media', 'warn', false, true),
    ('gore', 'alert', 'media', 'warn', false, true);

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
