# tiptoi-manager-tauri (inoffiziell)

Ein **inoffizieller, privater** Linux-Desktop-Client (Tauri + Vue), um einen
tiptoi(R)-Stift unter NixOS zu verwalten: Stift erkennen, Katalog nach
Kategorien durchsuchen und `.gme`-Audiodateien auf den Stift laden.

> ⚠️ **Dies ist kein offizielles Ravensburger-Produkt.** Siehe Abschnitt
> [Markenrechte / Trademark](#markenrechte--trademark) unten – bitte lesen,
> bevor du dieses Projekt weitergibst oder veröffentlichst.

## Markenrechte / Trademark

**„tiptoi“ ist eine eingetragene Marke der Ravensburger Verlag GmbH.**
Dieses Projekt:

- ist **nicht** von Ravensburger entwickelt, autorisiert, geprüft oder gesponsert,
- verwendet den Namen „tiptoi“ ausschließlich **beschreibend** (nominativer
  Gebrauch), um Kompatibilität zu einem bestehenden Produkt zu benennen,
- verwendet **keine** offiziellen Ravensburger-Logos oder -Markengrafiken
  (nur ein eigener, generischer Farbakzent in Orange),
- lädt Produktbilder/Katalogdaten zur Laufzeit direkt vom öffentlichen
  Ravensburger-Serviceportal (`service.ravensburger.de`) – es werden keine
  Ravensburger-Assets im Quellcode oder in Releases mitgeliefert.

Falls du dieses Projekt veröffentlichst/weitergibst: Verwende einen Namen, der
„tiptoi“ nicht als Markenbestandteil im App-Namen selbst führt (z. B. „Pen
Manager for Linux, kompatibel mit tiptoi®“), und weise immer deutlich auf die
fehlende Verbindung zu Ravensburger hin.

## Technischer & rechtlicher Hintergrund

Der Katalog-Client (`src-tauri/src/catalog.rs`) nutzt **ausschließlich
öffentlich erreichbare, unauthentifizierte** REST-Endpunkte des Ravensburger
Serviceportals (`service.ravensburger.de`, MindTouch/NICE CXone Expert) –
dieselben Endpunkte, die die Webseite selbst im Browser verwendet, um die
Seite "tiptoi® Audiodateien" aufzubauen:

- `GET /@api/deki/pages/{id}/subpages?format=json&limit=all` – listet
  Unterseiten (Kategorien/Produkte) einer Seite.
- `GET /@api/deki/pages/{id}/contents?format=json` – liefert den
  HTML-Inhalt einer Produktseite, daraus werden per Regex der
  `.gme`-Downloadlink und das Vorschaubild extrahiert.

Es werden **keine Zugangsdaten, Tokens, Client-Secrets oder private APIs**
mehr verwendet – anders als in einer früheren Version dieses Projekts, die
auf einer reverse-engineerten OAuth2-API basierte (mit Zugangsdaten, die mit
der offiziellen App geteilt waren). Die `.gme`-Dateien sind exakt dieselben,
die ein Nutzer auch manuell über die Serviceportal-Seite anklicken könnte:

<https://service.ravensburger.de/tiptoi%C2%AE/tiptoi%C2%AE_Audiodateien>

Trotzdem bleibt das Repo vorerst **privat** (siehe [LICENSE](./LICENSE)):
Die genutzten Endpunkte sind nicht offiziell als öffentliche API dokumentiert
oder stabil zugesichert und könnten sich jederzeit ändern; außerdem sind die
Nutzungsbedingungen des Serviceportals für automatisiertes Abfragen nicht
ausdrücklich geklärt.

### Offizieller Fallback

Falls sich die Serviceportal-Struktur ändert und der Katalog/Download in
dieser App nicht mehr funktioniert: `.gme`-Dateien lassen sich jederzeit
**ohne dieses Tool** manuell von derselben offiziellen Ravensburger-Seite
herunterladen und einfach in den `GME`-Ordner des Stifts kopieren.

## Features

- Automatische Erkennung eines angeschlossenen tiptoi-Stifts (Mountpunkt,
  Seriennummer, Firmware, Generation, Speicherplatz) ohne Wine/offizielle
  Software.
- Katalog-Navigation nach Kategorien (wie auf der Ravensburger-Seite): erst
  wird nur die Kategorie-Liste geladen (ein einzelner Request), Produkte
  einer Kategorie werden erst bei Auswahl nachgeladen und dann separat
  gecacht (24h, offline nutzbar solange der Cache frisch ist).
- `.gme`-Dateien direkt auf den Stift herunterladen, mit Live-Fortschritt.
- Stift sicher auswerfen.
- Eigenes, schlankes UI-Design (Tailwind CSS, kein PrimeVue/Fremd-UI-Kit –
  siehe [Technische Hinweise](#technische-hinweise)).

## Voraussetzungen (NixOS)

Rust-Toolchain + Tauri-Systembibliotheken müssen verfügbar sein. In diesem
Setup liegen sie deklarativ in `~/nixos-config/packages.nix`
(`rustc`, `pkg-config`, GTK3/WebKitGTK/AT-SPI/Pango/… Bibliotheken).

Für Build/Dev wird zusätzlich eine projekt-lokale `shell.nix` benötigt, da
`environment.systemPackages` keinen `PKG_CONFIG_PATH`/`LD_LIBRARY_PATH` für
Build-Tools bereitstellt:

```bash
nix-shell --run "npm install"
nix-shell --run "npx tauri dev"      # Entwicklung
nix-shell --run "npx tauri build"    # Release-Build
```

### Bekannte WebKitGTK/NixOS-Macken

Falls das Fenster weiß bleibt oder falsch rendert (häufig unter
Wayland-Sessions), hilft:

```bash
GDK_BACKEND=x11 WEBKIT_DISABLE_COMPOSITING_MODE=1 nix-shell --run "npx tauri dev"
```

`glib-networking` muss zusätzlich in `shell.nix` als `buildInput` vorhanden
sein (inkl. `GIO_EXTRA_MODULES`), sonst schlägt jeder HTTPS-Request des
WebViews mit „TLS support is not available“ fehl (z. B. Katalog-Thumbnails).

## Architektur

```
src/                    Vue 3 Frontend (Tailwind CSS, eigenes Design-System)
  components/
    AppSidebar.vue       Navigation (Stift/Katalog/Downloads/Hilfe)
    AppTopbar.vue         Seitentitel + Stift-Status
    PenScreen.vue         Stift-Infos, Speicherplatz, installierte Bücher
    CatalogScreen.vue     Kategorie-Navigation, Produktsuche, Download-Start
    DownloadScreen.vue    Live-Fortschritt laufender Downloads
    icons/Icon.vue         Eigenes, minimalistisches Inline-SVG-Icon-Set
  composables/useToast.js  Eigener Toast-Store (Ersatz für primevue/usetoast)

src-tauri/src/
  pen.rs      USB-Stift-Erkennung, .tiptoi.log-Parsing, GME-Header-Parsing,
              Speicherplatz (statvfs), sicheres Auswerfen (udisksctl/gio/umount)
  catalog.rs  Katalog-Client für das öffentliche Ravensburger Serviceportal
              (Kategorien, Produkt-Crawl pro Kategorie mit Disk-Cache),
              Datei-Download mit download-progress-Events
  lib.rs      Tauri-Commands: detect_pen, eject_pen, get_categories,
              get_category_catalog, download_book
```

## Technische Hinweise

- **Kein PrimeVue/Fremd-UI-Kit**: PrimeVue hat ab v5 (Juli 2026) auf eine
  kommerzielle Lizenz (PrimeUI) umgestellt und ist nicht mehr durchgehend
  MIT-lizenziert. Daher wurde die UI komplett selbst mit Tailwind CSS (MIT)
  gebaut – inklusive eigenem Button-/Card-/Progress-/Toast-„Design-System“ in
  `src/style.css` und einem eigenen Icon-Set.
- Die genaue Struktur der Serviceportal-API wurde anhand echter Antworten
  verifiziert: `<page.subpage id="…" subpages="true|false"><title>…</title>`
  für die Seitenhierarchie, `<a href="….gme">`/`<img src="…">` (HTML-entity-
  escaped, teils doppelt) im `contents`-Response für Download-Link und
  Vorschaubild.

## Lizenz

Siehe [LICENSE](./LICENSE) – kurz zusammengefasst: privater Eigenbedarf,
keine öffentliche Weitergabe/Veröffentlichung in aktueller Form.

## Releases / CI

Dieses Repo nutzt [release-please](https://github.com/googleapis/release-please)
(`.github/workflows/release-please.yml`): Commits nach
[Conventional Commits](https://www.conventionalcommits.org/) auf `main`
erzeugen automatisch eine Release-PR mit Changelog und Versionsbump
(`package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`). Wird
diese PR gemerged, entsteht ein Git-Tag + GitHub-Release, und ein zweiter Job
baut automatisch eine **AppImage** (`tauri build`, `bundle.targets:
["appimage"]`) und hängt sie als Release-Asset an.

> Hinweis: Da dieses Repo laut [LICENSE](./LICENSE) nicht öffentlich
> weitergegeben werden soll, nur in einem **privaten** GitHub-Repo verwenden
> – Releases/AppImages sind dann ausschließlich für dich persönlich
> (z. B. um sie auf einer zweiten Maschine zu installieren), nicht zur
> öffentlichen Verbreitung gedacht.
