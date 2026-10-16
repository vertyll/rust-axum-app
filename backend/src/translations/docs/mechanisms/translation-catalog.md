# Translation catalog

Where the text behind every message key comes from, and how an administrator's edits survive a deployment.

The defaults ship inside the binary, as ICU MessageFormat, from the files of their owners:

| Owner         | Files                                                                                                                                                             |
|---------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| each module   | `src/<module>/messages/pl.json` and `en.json`: the errors it answers with                                                                                         |
| `shared_http` | `src/shared_http/messages`: the errors every module shares                                                                                                        |
| the front-end | `translations/<area>/pl.json` and `en.json`: the texts of one area of its pages, such as `layout` (navigation, footer, error pages), `common`, `files` or `admin` |

Each module declares its files as a `Messages` constant, the type in `shared_kernel`, and the composition root collects
them, as it collects the modules' permissions; this module knows none of them. Both languages of an owner list the same
keys, and no key is shipped by two owners; the application does not start otherwise. At startup the stored catalog is
brought in line with them: new keys are added, changed defaults adopted and keys the code no longer uses dropped.

Anyone whose role grants `TRANSLATIONS_EDIT` can override any message; listing them all needs `TRANSLATIONS_VIEW`. An
override survives a new default until it is reset, and it is checked before it is stored: it must parse as ICU
MessageFormat and may use only the placeholders of its default, so a typo cannot break a message at runtime.

## Who reads it

Clients load one language from `GET /api/translations/{language}` and render every `code` of a problem document with
it. Adding an error to a module is therefore adding its key to both files of that module.
