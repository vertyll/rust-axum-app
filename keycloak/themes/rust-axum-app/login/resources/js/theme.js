// The reader's theme on Keycloak's pages: System follows the system, as Keycloak's own dark mode does; Light and Dark
// hold whatever the system says. The choice is kept in this browser for Keycloak's address. The first page of a
// sign-in may carry `ui_theme`, the theme the reader chose in the app, which then becomes the choice here too.
(() => {
  const KEY = "theme";
  const THEMES = ["system", "light", "dark"];
  const DARK_CLASS = "pf-v5-theme-dark";
  const systemDark = window.matchMedia("(prefers-color-scheme: dark)");

  // Taken once: the parameter leaves the address, so a reload keeps a choice made on this page.
  const address = new URL(window.location.href);
  const fromApp = address.searchParams.get("ui_theme");
  if (THEMES.includes(fromApp)) {
    store(fromApp);
    address.searchParams.delete("ui_theme");
    window.history.replaceState(window.history.state, "", address);
  }

  apply();
  systemDark.addEventListener("change", apply);
  document.addEventListener("DOMContentLoaded", apply);

  document.addEventListener("click", (event) => {
    const choice = event.target.closest("[data-theme-choice]");
    if (choice) {
      store(choice.dataset.themeChoice);
      apply();
      choice.closest("details")?.removeAttribute("open");
    }
  });

  document.addEventListener(
    "toggle",
    (event) => {
      const menu = event.target;
      if (menu.matches?.("[data-app-menu]") && menu.open) {
        for (const other of document.querySelectorAll("[data-app-menu][open]")) {
          if (other !== menu) {
            other.removeAttribute("open");
          }
        }
      }
    },
    true,
  );

  function chosen() {
    try {
      const theme = window.localStorage.getItem(KEY);
      return THEMES.includes(theme) ? theme : "system";
    } catch {
      return "system";
    }
  }

  function store(theme) {
    try {
      window.localStorage.setItem(KEY, theme);
    } catch {
      // A browser that keeps nothing still shows the choice until the next page.
    }
  }

  function apply() {
    const theme = chosen();
    const root = document.documentElement;
    root.dataset.theme = theme;
    root.classList.toggle(DARK_CLASS, theme === "dark" || (theme === "system" && systemDark.matches));
    document.querySelectorAll("[data-theme-choice]").forEach((button) => {
      const current = button.dataset.themeChoice === theme;
      button.setAttribute("aria-pressed", String(current));
      button.classList.toggle("kc-app-menu__item--active", current);
    });
  }
})();
