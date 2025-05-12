# TODO

- subscription_tokens table reset periodically?
- What to do if a user didn't get / accidentally deleted confirmation email. Currently once the user was sent a confirmation email,
  there is no mechanism for that email to be sent again.
  We could delete unconfirmed users daily, or send only 2 mails in a span of 24 hours max, or some combination of those options.
- Add hashing pepper. For dev and as a secret.
- add HMAC secret to fly.io before deploying
- HMAC secret rotation
