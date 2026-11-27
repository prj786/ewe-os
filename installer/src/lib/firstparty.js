// ewe's own opt-in apps — packages the [ewe] repository serves and nothing
// installs by default. The Apps screen offers them first, unchecked, in the
// same shape as Komble's curated tiles (curated.js is Komble's file, copied
// verbatim, and by its own rule does not list first-party apps). A pick is a
// plain "repo" app: the helper's `apps` verb installs it with pacman on the
// new system, where the ewe layer has already added [ewe].
//
// `plugin` names the first-party plugin that already brings the app: the
// Plugins screen offers ewe Safe as the *ewe Safe* plugin, whose manifest
// requires the ewe-safe package — picking it there installs the app. Such an
// app is offered here only when this ISO's payload has no such plugin, so
// nothing is offered twice. ewe Mobile has no plugin that requires it (the
// Phone plugin works without it and opens it where it is installed).
export const firstParty = [
  { name: "ewe Mobile", pkg: "ewe-mobile", source: "repo", plugin: null,
    desc: "Your phone on this computer." },
  { name: "ewe Safe", pkg: "ewe-safe", source: "repo", plugin: "ewe.safe",
    desc: "ewe's password manager." },
];

// What the Apps screen shows, given the Plugins screen's catalogue
// ([{id, …}], or null/[] when the payload offers no plugins).
export function firstPartyApps(catalog) {
  const offered = new Set((catalog || []).map((a) => a.id));
  return firstParty.filter((a) => !a.plugin || !offered.has(a.plugin));
}

export const firstPartyCategory = { id: "ewe", title: "From ewe" };
