# ruckchat-email

Transactional email sending for RuckChat, backed by the
[Postmark](https://postmarkapp.com/) API via the `postmark` crate. Used for
server-admin password resets and RocketChat migration credential delivery.
Sending is a no-op when no `EmailConfig` is supplied, so servers without
Postmark configured see no behavior change.
