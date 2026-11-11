// Opening one dropdown closes the others, as each is a <details> of its own,
// and opens its panel upwards when it does not fit below its trigger on the
// screen but does above, as near the end of a page. `toggle` does not bubble,
// so it is caught on its way down.
document.addEventListener(
  "toggle",
  (event) => {
    const menu = event.target;
    if (!(menu instanceof HTMLDetailsElement) || !menu.hasAttribute("data-dropdown") || !menu.open) {
      return;
    }
    for (const other of document.querySelectorAll("details[data-dropdown]")) {
      if (other !== menu) {
        other.open = false;
      }
    }
    const panel = menu.lastElementChild;
    panel.classList.remove("bottom-full", "mb-2");
    panel.classList.add("mt-2");
    const trigger = menu.getBoundingClientRect();
    const below = window.innerHeight - trigger.bottom;
    if (panel.offsetHeight > below && trigger.top > below) {
      panel.classList.remove("mt-2");
      panel.classList.add("bottom-full", "mb-2");
    }
  },
  true,
);
