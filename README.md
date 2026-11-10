# SchILD3 Updater

**Der passende SchILD3-Client zur SVWS-Serverversion, ohne manuelles Suchen nach dem richtigen Download.**

SchILD3 Updater ist ein Windows-Werkzeug für die Administration von SchILD3-Arbeitsplätzen. Es liest die Version Ihrer SVWS-Instanz, vergleicht sie mit dem installierten Client und sucht eine ausdrücklich dazu freigegebene Version in den offiziellen SchILD3-Releases. Ein angebotenes Update wird erst nach Ihrer Zustimmung heruntergeladen und installiert.

Der Updater ist in Rust geschrieben und wird als einzelne Windows-x64-EXE ausgeliefert. Eine Installation von Rust, einer WebView oder einer zusätzlichen VC-Runtime ist nicht erforderlich.

**[Downloads und Versionen](https://github.com/cschmidtoptima/Schild3Updater/releases)** | **[Fehler melden](https://github.com/cschmidtoptima/Schild3Updater/issues)** | **[MIT-Lizenz](LICENSE)**

## Auf einen Blick

- **Kompatibilität statt nur "neueste Version":** Es zählt die ausdrückliche Freigabe für die ermittelte SVWS-Serverversion. Kein automatisches Downgrade.
- **Prüfung beim Start:** Neue Client- und Updater-Versionen werden automatisch gesucht. Die Installation erfordert jeweils eine Bestätigung.
- **Lokale Ordner und Netzwerkfreigaben:** Der Clientpfad kann frei gewählt werden, auch als UNC-Pfad.
- **Dateien erhalten:** Nur Dateien aus dem Updatepaket werden ersetzt. Andere Dateien und Unterordner bleiben bestehen; betroffene Originale werden gesichert.
- **Geprüfte Downloads:** HTTPS, SHA-256-Prüfsummen und Versionskontrollen vor der Installation.
- **Portable Einstellungen:** Serveradresse und Clientpfad liegen in `settings.json` neben der EXE.
- **Selbstaktualisierung:** Der Updater kann seine eigene EXE aus den Releases dieses Repositorys aktualisieren.

Es werden keine SVWS-Server oder Datenbanken aktualisiert. Der Updater ist ein unabhängiges Hilfsprojekt und kein offizielles Produkt von SVWS-NRW. Produkt- und Firmennamen gehören ihren jeweiligen Rechteinhabern.

Das Projekt steht unter der liberalen **MIT-Lizenz**. Auch kommerzielle Nutzung, Änderungen und Weitergabe sind erlaubt; Lizenz- und Urheberrechtshinweise müssen erhalten bleiben. Details stehen im Abschnitt [Lizenz](#lizenz).

## Oberfläche

Die Oberfläche verwendet das eingebettete OPTIMA-Logo und einen anklickbaren Verweis auf <https://optima-computer.de/>. Das eigenständige Programm-Icon zeigt ein Dokument mit der Ziffer 3 und Aktualisierungspfeilen, nicht das Firmenlogo. Logo und Icons funktionieren auch offline. Größere Schrift, skalierbare Bedienicons und Per-Monitor-DPI-Unterstützung verbessern die Darstellung auf hochauflösenden Bildschirmen; soweit vorhanden wird die installierte Windows-Schrift Segoe UI verwendet. Unter **Info** stehen die Programmversion und die Lizenzhinweise der Bedienicons.

## Bedienung

1. `SchILD3Updater.exe` aus den [GitHub-Releases](https://github.com/cschmidtoptima/Schild3Updater/releases) herunterladen, in einen beschreibbaren Ordner legen und starten. Falls noch kein Release vorhanden ist, kann das Projekt wie unten beschrieben selbst gebaut werden.
2. HTTPS-Adresse der SVWS-Instanz eintragen.
3. Vorhandenen SchILD3-Clientordner mit `SchILD30.exe` wählen oder einen UNC-Pfad wie `\\server\freigabe\SchILD3` eingeben.
4. Einstellungen über das Speichersymbol rechts neben **Verbindung und Client** speichern. Die Datei `settings.json` wird direkt neben der Updater-EXE angelegt und von dort geladen, unabhängig vom Arbeitsverzeichnis. Die Einstellungen werden auch beim Start einer Prüfung gespeichert.
5. SchILD3 vor einem Update auf allen betroffenen Arbeitsplätzen schließen.
6. **Nur prüfen** zeigt die Versionen an. **Prüfen und aktualisieren** installiert bei Bedarf automatisch den passenden neueren Client.

Bei jedem Programmstart wird mit den gespeicherten Einstellungen einmal automatisch im Hintergrund auf eine passende neuere Clientversion geprüft. Ohne konfigurierte Serveradresse und Clientordner wird keine Prüfung gestartet. Fehler erscheinen im Hauptfenster und im Protokoll.

Ist ein Update verfügbar, erscheint ein Dialog mit Serverversion sowie installierter und angebotener Clientversion. **OK** startet den Download und die Installation dieser Version. **Abbrechen**, Escape oder das Schließen des Dialogs verwirft das Angebot, ohne Clientdateien zu verändern. Bei bereits passendem Client erscheint kein Update-Dialog. Vor der Installation werden Server- und Clientversion erneut geprüft.

Das Programm führt keine unbeaufsichtigten Hintergrundupdates aus und startet SchILD3 nicht automatisch. Bei UNC-Pfaden gelten die Netzwerkberechtigungen des angemeldeten Windows-Benutzers. Zugangsdaten werden nicht gespeichert. Schreibrechte im Updater-Ordner zum Speichern der Einstellungen und im gesamten betroffenen Clientordner zum Aktualisieren sind erforderlich. Eine bisher unter `%APPDATA%\SchILD3Updater\settings.json` gespeicherte Konfiguration wird nicht automatisch übernommen; sie kann bei Bedarf neben die Updater-EXE verschoben werden.

Vor dem ersten produktiven Einsatz mit einer Testkopie prüfen und eine unabhängige Sicherung des Clientordners erstellen. Die integrierte Sicherung ersetzt kein vollständiges Backup.

## Updater-Version und Selbstaktualisierung

Die Programmversion wird zentral in `Cargo.toml` gepflegt (aktuell **0.2.0**). Oberfläche, Windows-Dateiversion, Produktversion und Manifest übernehmen diese Version beim Build.

Bei jedem Start prüft der Updater zunächst den neuesten stabilen Release unter <https://github.com/cschmidtoptima/Schild3Updater/releases>. Das funktioniert auch ohne konfigurierte SchILD3-Verbindung. Eine neuere Version wird mit eigenem Dialog angeboten: **OK** lädt die EXE und startet nach dem Austausch neu; **Abbrechen**, Escape oder Schließen des Dialogs lässt die EXE unverändert. Danach folgt die bisherige Clientprüfung. Unter **Info > Auf Updater-Update prüfen** kann die Prüfung erneut gestartet werden. Netzwerk-/GitHub-Fehler werden protokolliert und blockieren die Clientprüfung nicht.

Nur stabile SemVer-Tags wie `v0.3.0` und das Release-Asset **SchILD3Updater.exe** aus diesem Repository werden akzeptiert. Gleich alte oder ältere Versionen, Entwürfe und Vorabversionen werden nicht installiert. Vor dem Austausch werden HTTPS-Download, Größenlimit, SHA-256-Digest aus der GitHub-API und die eingebettete Windows-Dateiversion geprüft. Der Windows-Austausch erfolgt mit `self-replace`; Einstellungen und Clientdateien werden nicht angefasst. Schreibrechte im Updater-Ordner sind erforderlich. Keine automatische Rechteerhöhung. Bei einem Stromausfall während des Austauschs kann eine manuelle Wiederherstellung erforderlich sein.

Das Repository und seine Releases müssen **öffentlich erreichbar** sein. Private Repositorys ohne Anmeldung sowie fehlende Releases liefern GitHub 404; Zugangstokens werden bewusst nicht im Programm hinterlegt. Ältere Updater ohne Selbstaktualisierung müssen einmal manuell durch Version 0.2.0 oder neuer ersetzt werden.

### Neue Version veröffentlichen

Der Workflow `.github/workflows/release.yml` baut und testet Windows x64 automatisch, sobald ein passender Versionstag gepusht wird. Er prüft Tag und Windows-Dateiversion und veröffentlicht **SchILD3Updater.exe**, **SHA256SUMS.txt**, **LICENSE**, **README.md** und **lucide-LICENSE** als GitHub-Release-Assets. GitHub Actions muss im Repository aktiviert sein.

1. `version` in `Cargo.toml` erhöhen, z.B. auf `0.3.0`.
2. `cargo test` ausführen. Dabei wird auch `Cargo.lock` aktualisiert.
3. Quellcode, Workflow und beide Cargo-Dateien committen und pushen.
4. Den passenden Tag erstellen und pushen:

```powershell
git tag v0.3.0
git push origin v0.3.0
```

Für den ersten Release dieses Standes stattdessen `v0.2.0` verwenden. Erst nach erfolgreichem Workflow und veröffentlichtem Release können installierte Updater die Datei herunterladen. Die Prüfsumme in der GitHub-API kann kurz nach dem Upload noch fehlen; in diesem Fall wird das Update bis zur erneuten Prüfung nicht angeboten. Die EXE ist weiterhin nicht digital signiert; SHA-256 ersetzt keine Herausgebersignatur und schützt nicht gegen ein kompromittiertes GitHub-Konto.

## Client-Versionsauswahl

- Serverversion: öffentlicher SVWS-Endpunkt `status/version` relativ zur konfigurierten Instanzadresse. HTTPS-Zertifikate werden normal validiert.
- Clientversion: Windows-Dateiversion von `SchILD30.exe`, die ersten drei Versionsbestandteile.
- Quelle: <https://github.com/SVWS-NRW/Schild-NRW-3/releases/>, ohne Entwurfs- und Vorabversionen.
- Es zählen ausschließlich eindeutige `Lauffähig mit ... SVWS-Server ...`-Freigaben in den Releasehinweisen, mit exakter Serverversion. Eine nur erwähnte Version oder eine unbekannte Formulierung reicht nicht.
- Aus passenden Releases wird die höchste stabile Clientversion mit `Setup_SchILD3_vVERSION.zip` gewählt. Keine automatische Rückstufung und kein Raten anhand ähnlicher Versionsnummern.
- Vor der Installation werden der SHA-256-Digest aus der GitHub-API, die EXE-Version im ZIP und `server.json` des Pakets geprüft. Fehlende Prüfsummen oder Widersprüche sperren das Update.
- GitHub begrenzt anonyme API-Anfragen; bei HTTP 403/429 später erneut versuchen.

## Dateierhalt und Sicherung

Nur im ZIP enthaltene Dateien werden ersetzt oder hinzugefügt. Andere Dateien und Unterordner werden nicht gelöscht. Auch lokale Konfigurationsdateien werden ersetzt, falls sie im ZIP vorkommen.

Vor jeder Installation werden alle zu ersetzenden Dateien unter `.schild3-updater-backups\update-...\originals` im Clientordner gesichert. `manifest.json` dokumentiert die Paketdateien und zuvor vorhandene Dateien. Sicherungen bleiben erhalten und können nach eigener Prüfung manuell gelöscht werden. Das Programm benötigt Platz für Download, entpacktes Paket, Sicherung und temporäre Ersatzdateien.

Die Dateien werden einzeln über temporäre Dateien im jeweiligen Zielordner ersetzt. Bei einem normalen Schreib- oder Verifikationsfehler werden ersetzte Dateien zurückgesichert und neu angelegte Dateien entfernt. Bei unvollständiger Rücksetzung wird der Sicherungspfad angezeigt. Eine Sperrdatei verhindert parallele Updates durch dieses Programm; gesperrte Paketdateien werden vorab geprüft. Junctions und symbolische Links in betroffenen Pfaden werden abgewiesen.

Ein Dateibaum auf einer Netzwerkfreigabe kann nicht als Ganzes atomar aktualisiert werden. Stromausfall, Prozessabbruch oder Verbindungsabbruch können eine manuelle Wiederherstellung erfordern. Eine Sicherung mit Manifest ohne `completed`- oder `rolled-back`-Marker sperrt weitere Prüfungen/Updates. In diesem Fall SchILD3 geschlossen halten, die Dateien aus `originals` an ihre bisherigen relativen Pfade zurückkopieren und die im Manifest aufgeführten, vorher nicht vorhandenen Dateien entfernen. Erst nach vollständiger Wiederherstellung einen `rolled-back`-Marker in diesem Sicherungsordner anlegen oder die geprüfte Sicherung aus dem Sicherungsverzeichnis verschieben. Es werden keine Server- oder Datenbankupdates ausgeführt.

## Selbst bauen

Voraussetzungen: Rust stable mit MSVC-Toolchain und Visual Studio C++ Build Tools inkl. Windows SDK.

```powershell
cargo test
cargo build --release
```

Ergebnis: `target\x86_64-pc-windows-msvc\release\schild3-updater.exe`. Die C-Runtime wird statisch gelinkt. Standard-Windows-Systembibliotheken und Grafiktreiber werden weiterhin verwendet. Die EXE ist nicht digital signiert; Windows SmartScreen kann deshalb eine Warnung anzeigen.

Nur lesende Prüfung mit der Debug-EXE in einer Konsole:

```powershell
cargo run -- --check https://schule.example.org/ C:\SchILD3
```

Automatisierte Tests decken Versionszuordnung, Ablehnung unsicherer URLs und Archivpfade, Dateierhalt, Sicherung, Rücksetzung und Windows-Dateisperren ab. Eine reale Schulinstallation wird durch die Tests nicht verändert.

Ein optionaler Netzwerktest verwendet ein lokal heruntergeladenes älteres offizielles ZIP als Ausgangsversion. Er aktualisiert ausschließlich einen temporären Ordner, prüft den SHA-256-gesicherten Download, die neue EXE-Version, die Sicherung und den Erhalt lokaler Dateien:

```powershell
$env:SCHILD3_TEST_ARCHIVE = 'C:\Downloads\Setup_SchILD3_v3.5.0.zip'
$env:SCHILD3_TEST_SERVER = 'https://schule.example.org/'
cargo test official_release_end_to_end -- --ignored --nocapture
```

## Lizenz

Der eigene Projektcode steht unter der **[MIT-Lizenz](LICENSE)** (SPDX: `MIT`), einer liberalen, OSI-anerkannten Open-Source-Lizenz.

- Private und kommerzielle Nutzung sind erlaubt.
- Änderung, Weitergabe und Verkauf sind erlaubt, auch als Teil proprietärer Software.
- Der Urheberrechtsvermerk und der Lizenztext müssen bei Kopien oder wesentlichen Teilen der Software erhalten bleiben.
- Es besteht keine Pflicht, eigene Änderungen zu veröffentlichen.
- Die Software wird im gesetzlich zulässigen Umfang ohne Gewährleistung bereitgestellt.

Maßgeblich ist der englische Lizenztext. Die Projektlizenz ändert nicht die Lizenzen der eingebundenen Rust-Abhängigkeiten oder der Lucide-/Feather-Icons. Grafikquellen stehen in [assets/README.md](assets/README.md), der mitgelieferte Icon-Lizenztext in [assets/lucide-LICENSE](assets/lucide-LICENSE). Marken- und Kennzeichenrechte an Namen und Firmenlogos werden durch die MIT-Lizenz nicht eingeräumt.

## Mitwirken und Fehler melden

Fehlerberichte und Verbesserungsvorschläge sind über [GitHub Issues](https://github.com/cschmidtoptima/Schild3Updater/issues) willkommen. Bitte Windows-Version, Updater-Version, die relevanten Versionsnummern und eine nachvollziehbare Fehlerbeschreibung angeben. Protokolle vor dem Hochladen auf Schulnamen, interne Serveradressen, UNC-Pfade und personenbezogene Daten prüfen und diese Angaben entfernen. Keine Schülerdaten, Zugangsdaten oder kompletten Clientinstallationen hochladen.

Für Codebeiträge bitte einen Pull Request erstellen, bestehende Tests berücksichtigen und `cargo fmt --check` sowie `cargo test --locked` ausführen. Bitte nur Code einreichen, für den Sie die erforderlichen Rechte besitzen und den Sie unter der MIT-Lizenz bereitstellen dürfen.

## Veröffentlichung vorbereiten

- Repository-Beschreibung für das GitHub-About-Feld: **Windows-Updater für SchILD3 mit SVWS-Kompatibilitätsprüfung, Sicherungen und GitHub-Selbstaktualisierung. Open Source unter MIT.**
- Repository und Releases öffentlich erreichbar machen und GitHub Actions aktivieren. Vorher Rechte am eigenen Code, Firmenlogo und sonstigen eingebundenen Inhalten prüfen.
- Lokale Einstellungen, Buildordner und EXE-Dateien gehören nicht in den Quellcode-Commit. Die Ignore-Regeln gelten nicht rückwirkend für bereits versionierte Dateien; diese müssen aus dem Git-Index entfernt werden. Lokale Dateien können dabei erhalten bleiben.
- Vor einer öffentlichen Freigabe auch die Git-Historie auf interne Informationen prüfen: Das Entfernen aus dem aktuellen Stand entfernt frühere Versionen nicht. Eine eventuell erforderliche Historienbereinigung muss gesondert geplant werden.
- Lizenz und Hinweise zu Abhängigkeiten bei der Weitergabe beilegen. Der Release-Workflow stellt die Projektlizenz, README und Icon-Lizenz neben der EXE bereit. Für die Rust-Abhängigkeiten gelten zusätzlich deren eigene Lizenz- und Hinweispflichten; vor einer Binärveröffentlichung prüfen und erforderliche Hinweise beilegen.
- Änderungen committen und pushen; danach wie oben beschrieben einen zur Cargo-Version passenden Tag veröffentlichen. Es werden keine Schulkonfigurationen in den Release aufgenommen.