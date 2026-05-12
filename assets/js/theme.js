export const setTheme = (theme, {document, localStorage}) => {
  if (theme === "system") {
    localStorage.removeItem("phx:theme")
    document.documentElement.removeAttribute("data-theme")
  } else {
    localStorage.setItem("phx:theme", theme)
    document.documentElement.setAttribute("data-theme", theme)
  }
}

export const installThemeHandlers = ({document, localStorage, window}) => {
  if (!document.documentElement.hasAttribute("data-theme")) {
    setTheme(localStorage.getItem("phx:theme") || "system", {document, localStorage})
  }

  window.addEventListener("storage", (e) => {
    if (e.key === "phx:theme") {
      setTheme(e.newValue || "system", {document, localStorage})
    }
  })

  window.addEventListener("phx:set-theme", (e) => {
    setTheme(e.target.dataset.phxTheme, {document, localStorage})
  })
}
