# Error responses

What the API answers when it refuses a request, and how a client turns that into text.

Every refusal is an RFC 9457 problem document (`application/problem+json`, built in
`shared_infrastructure/problem.rs`). The server never sends prose a person reads:

| Field                     | Holds                                                                 |
|---------------------------|-----------------------------------------------------------------------|
| `type`, `title`, `status` | `about:blank`, the status's reason phrase, the HTTP status            |
| `code`                    | a key of the translation catalog, e.g. `errors.not_found`             |
| `detail`                  | the same key as `code`                                                |
| `args`                    | the ICU arguments for that key; in a validation error, keyed by field |
| `errors`                  | in a validation error, the message keys of each invalid field         |

The client translates: it loads the catalog from `GET /api/translations/{language}` and formats `code` with `args` as
an ICU MessageFormat message, in its reader's language. A new error is therefore a new key in `translations/en.json`
and `pl.json`, never a sentence in the code. Each module maps its domain errors to keys in
`infrastructure/http/error.rs`.
