import assert from "node:assert/strict"
import test from "node:test"

import {installThemeHandlers, setTheme} from "../js/theme.js"

const buildFakes = () => {
  const attributes = new Map()
  const storage = new Map()
  const handlers = new Map()

  return {
    document: {
      documentElement: {
        hasAttribute(name) {
          return attributes.has(name)
        },
        getAttribute(name) {
          return attributes.get(name)
        },
        setAttribute(name, value) {
          attributes.set(name, value)
        },
        removeAttribute(name) {
          attributes.delete(name)
        },
      },
    },
    localStorage: {
      getItem(key) {
        return storage.has(key) ? storage.get(key) : null
      },
      setItem(key, value) {
        storage.set(key, value)
      },
      removeItem(key) {
        storage.delete(key)
      },
    },
    window: {
      addEventListener(type, handler) {
        const currentHandlers = handlers.get(type) || []
        handlers.set(type, [...currentHandlers, handler])
      },
      dispatchEvent(type, event) {
        for (const handler of handlers.get(type) || []) {
          handler(event)
        }
      },
    },
  }
}

test("system theme clears persisted and explicit document theme", () => {
  const fakes = buildFakes()

  fakes.localStorage.setItem("phx:theme", "dark")
  fakes.document.documentElement.setAttribute("data-theme", "dark")

  setTheme("system", fakes)

  assert.equal(fakes.localStorage.getItem("phx:theme"), null)
  assert.equal(fakes.document.documentElement.hasAttribute("data-theme"), false)
})

test("explicit theme is persisted and applied to the document", () => {
  const fakes = buildFakes()

  setTheme("light", fakes)

  assert.equal(fakes.localStorage.getItem("phx:theme"), "light")
  assert.equal(fakes.document.documentElement.getAttribute("data-theme"), "light")
})

test("installing theme handlers applies persisted/default theme and responds to theme events", () => {
  const fakes = buildFakes()

  fakes.localStorage.setItem("phx:theme", "dark")
  installThemeHandlers(fakes)

  assert.equal(fakes.document.documentElement.getAttribute("data-theme"), "dark")

  fakes.window.dispatchEvent("storage", {key: "phx:theme", newValue: "light"})
  assert.equal(fakes.localStorage.getItem("phx:theme"), "light")
  assert.equal(fakes.document.documentElement.getAttribute("data-theme"), "light")

  fakes.window.dispatchEvent("phx:set-theme", {target: {dataset: {phxTheme: "system"}}})
  assert.equal(fakes.localStorage.getItem("phx:theme"), null)
  assert.equal(fakes.document.documentElement.hasAttribute("data-theme"), false)

  const defaultFakes = buildFakes()
  installThemeHandlers(defaultFakes)

  assert.equal(defaultFakes.localStorage.getItem("phx:theme"), null)
  assert.equal(defaultFakes.document.documentElement.hasAttribute("data-theme"), false)
})
