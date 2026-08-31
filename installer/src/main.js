import { mount } from "svelte";
import App from "./App.svelte";
import "./app.css";

// Same look as every ewe app: the shared app.css tokens only fire under the
// .dark class (ewe-settings does the same), and the accent pins to the DE's
// default — the installed desktop opens on exactly this blue.
document.documentElement.classList.add("dark");
document.documentElement.style.setProperty("--accent", "#0a84ff");

// Svelte 5: components are not classes — `new App(...)` throws and leaves a
// blank window. mount() is the v5 entry point.
const app = mount(App, { target: document.getElementById("app") });

export default app;
