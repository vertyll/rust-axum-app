# translations

Owns the text clients show: the catalogue every error key and label resolves against, in Polish and English.

## Defaults and overrides

The defaults ship inside the binary (`translations/pl.json` and `en.json`, ICU MessageFormat); both files must list the
same keys. At startup the stored catalogue is brought in line with them: new keys are added, changed defaults adopted
and keys the code no longer uses dropped.

An admin can override any message. An override survives a new default until it is reset, and it is checked before it
is stored: it must parse as ICU MessageFormat and may use only the placeholders of its default, so a typo cannot break
a message at runtime.

## Who reads it

Clients load one language from `GET /api/translations/{language}` and render every `code` of a problem document with
it. Adding an error anywhere in the application is therefore adding its key to both files here.

## Ports

| Port                    | Adapter                       | What it does         |
|-------------------------|-------------------------------|----------------------|
| `TranslationRepository` | `ToastyTranslationRepository` | the stored catalogue |
