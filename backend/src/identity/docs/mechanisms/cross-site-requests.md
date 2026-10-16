# Cross-site requests

Why a forged request from another site cannot act with the user's session. CSRF tokens are not needed.

The cookie is `SameSite=Lax`, which keeps it off cross-site writes. The middleware adds a second check: an unsafe
request whose `Sec-Fetch-Site` is neither `same-origin` nor `none` is not given the session's token, and a logout from
another site is refused with `403`. CSRF tokens are therefore not used.
