# translations

Owns the text clients show: the catalog every error key and label resolves against, in Polish and English.

## Ports

| Port                    | Adapter                       | What it does       |
|-------------------------|-------------------------------|--------------------|
| `TranslationRepository` | `ToastyTranslationRepository` | the stored catalog |

## Mechanisms

- [Translation catalog](docs/mechanisms/translation-catalog.md) – Where the text behind every message key comes from,
  and how an administrator's edits survive a deployment.
