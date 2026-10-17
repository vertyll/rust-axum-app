<#-- The frame of every e-mail, drawn as the app draws its cards: light by default, and dark where the mail client
     follows a dark system. Mail clients read neither oklch() nor light-dark(), so the app's tokens are written here
     as hex. -->
<#macro emailLayout>
<!DOCTYPE html>
<html lang="${locale.language}" dir="${(ltr)?then('ltr','rtl')}">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <meta name="color-scheme" content="light dark">
    <meta name="supported-color-schemes" content="light dark">
    <style>
        .text a { color: #1c1f25; text-decoration: underline; }
        @media (prefers-color-scheme: dark) {
            .page { background: #0c1015 !important; }
            .card { background: #171b20 !important; border-color: #2a2e34 !important; }
            .text, .text a { color: #e5e8ec !important; }
            .muted { color: #9499a0 !important; }
        }
    </style>
</head>
<body class="page" style="margin: 0; padding: 0; background: #fafbfd;">
<table role="presentation" width="100%" cellpadding="0" cellspacing="0" class="page" style="background: #fafbfd;">
    <tr>
        <td align="center" style="padding: 32px 16px;">
            <table role="presentation" width="100%" cellpadding="0" cellspacing="0" style="max-width: 560px;">
                <tr>
                    <td class="text" style="padding: 0 4px 16px; font-family: Geist, ui-sans-serif, system-ui, -apple-system, Segoe UI, Roboto, Helvetica, Arial, sans-serif; font-size: 20px; font-weight: 700; color: #1c1f25;">${realmName}</td>
                </tr>
                <tr>
                    <td class="card text" style="background: #fcfdff; border: 1px solid #dbdee3; border-radius: 12px; padding: 24px; font-family: Geist, ui-sans-serif, system-ui, -apple-system, Segoe UI, Roboto, Helvetica, Arial, sans-serif; font-size: 14px; line-height: 1.6; color: #1c1f25;">
                        <#nested>
                    </td>
                </tr>
            </table>
        </td>
    </tr>
</table>
</body>
</html>
</#macro>
