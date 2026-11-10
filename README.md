# SchILD3 Updater

**Der passende SchILD3-Client zur SVWS-Serverversion, ohne manuelles Suchen nach dem richtigen Download.**

SchILD3 Updater ist ein Windows-Werkzeug fuer die Administration von SchILD3-Arbeitsplaetzen. Es liest die Version Ihrer SVWS-Instanz, vergleicht sie mit dem installierten Client und sucht eine ausdruecklich dazu freigegebene Version in den offiziellen SchILD3-Releases. Ein angebotenes Update wird erst nach Ihrer Zustimmung heruntergeladen und installiert.

Der Updater ist in Rust geschrieben und wird als einzelne Windows-x64-EXE ausgeliefert. Eine Installation von Rust, einer WebView oder einer zusaetzlichen VC-Runtime ist nicht erforderlich.

**[Downloads und Versionen](https://github.com/cschmidtoptima/Schild3Updater/releases)** | **[Fehler melden](https://github.com/cschmidtoptima/Schild3Updater/issues)** | **[MIT-Lizenz](LICENSE)**

## Auf einen Blick

- **Kompatibilitaet statt nur "neueste Version":** Es zaehlt die ausdrueckliche Freigabe fuer die ermittelte SVWS-Serverversion. Kein automatisches Downgrade.
- **Pruefung beim Start:** Neue Client- und Updater-Versionen werden automatisch gesucht. Die Installation erfordert jeweils eine Bestaetigung.
- **Lokale Ordner und Netzwerkfreigaben:** Der Clientpfad kann frei gewaehlt werden, auch als UNC-Pfad.
- **Dateien erhalten:** Nur Dateien aus dem Updatepaket werden ersetzt. Andere Dateien und Unterordner bleiben bestehen; betroffene Originale werden gesichert.
- **Gepruefte Downloads:** HTTPS, SHA-256-Pruefsummen und Versionskontrollen vor der Installation.
- **Portable Einstellungen:** Serveradresse und Clientpfad liegen in `settings.json` neben der EXE.
- **Selbstaktualisierung:** Der Updater kann seine eigene EXE aus den Releases dieses Repositorys aktualisieren.

Es werden keine SVWS-Server oder Datenbanken aktualisiert. Der Updater ist ein unabhaengiges Hilfsprojekt und kein offizielles Produkt von SVWS-NRW. Produkt- und Firmennamen gehoeren ihren jeweiligen Rechteinhabern.

Das Projekt steht unter der liberalen **MIT-Lizenz**. Auch kommerzielle Nutzung, Aenderungen und Weitergabe sind erlaubt; Lizenz- und Urheberrechtshinweise muessen erhalten bleiben. Details stehen im Abschnitt [Lizenz](#lizenz).

## Oberflaeche

Die Oberflaeche verwendet das eingebettete OPTIMA-Logo und einen anklickbaren Verweis auf <https://optima-computer.de/>. Das eigenstaendige Programm-Icon zeigt ein Dokument mit der Ziffer 3 und Aktualisierungspfeilen, nicht das Firmenlogo. Logo und Icons funktionieren auch offline. Groessere Schrift, skalierbare Bedienicons und Per-Monitor-DPI-Unterstuetzung verbessern die Darstellung auf hochaufloesenden Bildschirmen; soweit vorhanden wird die installierte Windows-Schrift Segoe UI verwendet. Unter **Info** stehen die Programmversion und die Lizenzhinweise der Bedienicons.

## Bedienung

1. `SchILD3Updater.exe` aus den [GitHub-Releases](https://github.com/cschmidtoptima/Schild3Updater/releases) herunterladen, in einen beschreibbaren Ordner legen und starten. Falls noch kein Release vorhanden ist, kann das Projekt wie unten beschrieben selbst gebaut werden.
2. HTTPS-Adresse der SVWS-Instanz eintragen.
3. Vorhandenen SchILD3-Clientordner mit `SchILD30.exe` waehlen oder einen UNC-Pfad wie `\\server\freigabe\SchILD3` eingeben.
4. Einstellungen ueber das Speichersymbol rechts neben **Verbindung und Client** speichern. Die Datei `settings.json` wird direkt neben der Updater-EXE angelegt und von dort geladen, unabhaengig vom Arbeitsverzeichnis. Die Einstellungen werden auch beim Start einer Pruefung gespeichert.
5. SchILD3 vor einem Update auf allen betroffenen Arbeitsplaetzen schliessen.
6. **Nur pruefen** zeigt die Versionen an. **Pruefen und aktualisieren** installiert bei Bedarf automatisch den passenden neueren Client.

Bei jedem Programmstart wird mit den gespeicherten Einstellungen einmal automatisch im Hintergrund auf eine passende neuere Clientversion geprueft. Ohne konfigurierte Serveradresse und Clientordner wird keine Pruefung gestartet. Fehler erscheinen im Hauptfenster und im Protokoll.

Ist ein Update verfuegbar, erscheint ein Dialog mit Serverversion sowie installierter und angebotener Clientversion. **OK** startet den Download und die Installation dieser Version. **Abbrechen**, Escape oder das Schliessen des Dialogs verwirft das Angebot, ohne Clientdateien zu veraendern. Bei bereits passendem Client erscheint kein Update-Dialog. Vor der Installation werden Server- und Clientversion erneut geprueft.

Das Programm fuehrt keine unbeaufsichtigten Hintergrundupdates aus und startet SchILD3 nicht automatisch. Bei UNC-Pfaden gelten die Netzwerkberechtigungen des angemeldeten Windows-Benutzers. Zugangsdaten werden nicht gespeichert. Schreibrechte im Updater-Ordner zum Speichern der Einstellungen und im gesamten betroffenen Clientordner zum Aktualisieren sind erforderlich. Eine bisher unter `%APPDATA%\SchILD3Updater\settings.json` gespeicherte Konfiguration wird nicht automatisch uebernommen; sie kann bei Bedarf neben die Updater-EXE verschoben werden.

Vor dem ersten produktiven Einsatz mit einer Testkopie pruefen und eine unabhaengige Sicherung des Clientordners erstellen. Die integrierte Sicherung ersetzt kein vollstaendiges Backup.

## Updater-Version und Selbstaktualisierung

Die Programmversion wird zentral in `Cargo.toml` gepflegt (aktuell **0.2.0**). Oberflaeche, Windows-Dateiversion, Produktversion und Manifest uebernehmen diese Version beim Build.

Bei jedem Start prueft der Updater zunaechst den neuesten stabilen Release unter <https://github.com/cschmidtoptima/Schild3Updater/releases>. Das funktioniert auch ohne konfigurierte SchILD3-Verbindung. Eine neuere Version wird mit eigenem Dialog angeboten: **OK** laedt die EXE und startet nach dem Austausch neu; **Abbrechen**, Escape oder Schliessen des Dialogs laesst die EXE unveraendert. Danach folgt die bisherige Clientpruefung. Unter **Info > Auf Updater-Update pruefen** kann die Pruefung erneut gestartet werden. Netzwerk-/GitHub-Fehler werden protokolliert und blockieren die Clientpruefung nicht.

Nur stabile SemVer-Tags wie `v0.3.0` und das Release-Asset **SchILD3Updater.exe** aus diesem Repository werden akzeptiert. Gleich alte oder aeltere Versionen, Entwuerfe und Vorabversionen werden nicht installiert. Vor dem Austausch werden HTTPS-Download, Groessenlimit, SHA-256-Digest aus der GitHub-API und die eingebettete Windows-Dateiversion geprueft. Der Windows-Austausch erfolgt mit `self-replace`; Einstellungen und Clientdateien werden nicht angefasst. Schreibrechte im Updater-Ordner sind erforderlich. Keine automatische Rechteerhoehung. Bei einem Stromausfall waehrend des Austauschs kann eine manuelle Wiederherstellung erforderlich sein.

Das Repository und seine Releases muessen **oeffentlich erreichbar** sein. Private Repositorys ohne Anmeldung sowie fehlende Releases liefern GitHub 404; Zugangstokens werden bewusst nicht im Programm hinterlegt. Aeltere Updater ohne Selbstaktualisierung muessen einmal manuell durch Version 0.2.0 oder neuer ersetzt werden.

### Neue Version veroeffentlichen

Der Workflow `.github/workflows/release.yml` baut und testet Windows x64 automatisch, sobald ein passender Versionstag gepusht wird. Er prueft Tag und Windows-Dateiversion und veroeffentlicht **SchILD3Updater.exe**, **SHA256SUMS.txt**, **LICENSE**, **README.md** und **lucide-LICENSE** als GitHub-Release-Assets. GitHub Actions muss im Repository aktiviert sein.

1. `version` in `Cargo.toml` erhoehen, z.B. auf `0.3.0`.
2. `cargo test` ausfuehren. Dabei wird auch `Cargo.lock` aktualisiert.
3. Quellcode, Workflow und beide Cargo-Dateien committen und pushen.
4. Den passenden Tag erstellen und pushen:

```powershell
git tag v0.3.0
git push origin v0.3.0
```

Fuer den ersten Release dieses Standes stattdessen `v0.2.0` verwenden. Erst nach erfolgreichem Workflow und veroeffentlichtem Release koennen installierte Updater die Datei herunterladen. Die Pruefsumme in der GitHub-API kann kurz nach dem Upload noch fehlen; in diesem Fall wird das Update bis zur erneuten Pruefung nicht angeboten. Die EXE ist weiterhin nicht digital signiert; SHA-256 ersetzt keine Herausgebersignatur und schuetzt nicht gegen ein kompromittiertes GitHub-Konto.

## Client-Versionsauswahl

- Serverversion: oeffentlicher SVWS-Endpunkt `status/version` relativ zur konfigurierten Instanzadresse. HTTPS-Zertifikate werden normal validiert.
- Clientversion: Windows-Dateiversion von `SchILD30.exe`, die ersten drei Versionsbestandteile.
- Quelle: <https://github.com/SVWS-NRW/Schild-NRW-3/releases/>, ohne Entwurfs- und Vorabversionen.
- Es zaehlen ausschliesslich eindeutige `Lauffaehig mit ... SVWS-Server ...`-Freigaben in den Releasehinweisen, mit exakter Serverversion. Eine nur erwaehnte Version oder eine unbekannte Formulierung reicht nicht.
- Aus passenden Releases wird die hoechste stabile Clientversion mit `Setup_SchILD3_vVERSION.zip` gewaehlt. Keine automatische Rueckstufung und kein Raten anhand aehnlicher Versionsnummern.
- Vor der Installation werden der SHA-256-Digest aus der GitHub-API, die EXE-Version im ZIP und `server.json` des Pakets geprueft. Fehlende Pruefsummen oder Widersprueche sperren das Update.
- GitHub begrenzt anonyme API-Anfragen; bei HTTP 403/429 spaeter erneut versuchen.

## Dateierhalt und Sicherung

Nur im ZIP enthaltene Dateien werden ersetzt oder hinzugefuegt. Andere Dateien und Unterordner werden nicht geloescht. Auch lokale Konfigurationsdateien werden ersetzt, falls sie im ZIP vorkommen.

Vor jeder Installation werden alle zu ersetzenden Dateien unter `.schild3-updater-backups\update-...\originals` im Clientordner gesichert. `manifest.json` dokumentiert die Paketdateien und zuvor vorhandene Dateien. Sicherungen bleiben erhalten und koennen nach eigener Pruefung manuell geloescht werden. Das Programm benoetigt Platz fuer Download, entpacktes Paket, Sicherung und temporaere Ersatzdateien.

Die Dateien werden einzeln ueber temporaere Dateien im jeweiligen Zielordner ersetzt. Bei einem normalen Schreib- oder Verifikationsfehler werden ersetzte Dateien zurueckgesichert und neu angelegte Dateien entfernt. Bei unvollstaendiger Ruecksetzung wird der Sicherungspfad angezeigt. Eine Sperrdatei verhindert parallele Updates durch dieses Programm; gesperrte Paketdateien werden vorab geprueft. Junctions und symbolische Links in betroffenen Pfaden werden abgewiesen.

Ein Dateibaum auf einer Netzwerkfreigabe kann nicht als Ganzes atomar aktualisiert werden. Stromausfall, Prozessabbruch oder Verbindungsabbruch koennen eine manuelle Wiederherstellung erfordern. Eine Sicherung mit Manifest ohne `completed`- oder `rolled-back`-Marker sperrt weitere Pruefungen/Updates. In diesem Fall SchILD3 geschlossen halten, die Dateien aus `originals` an ihre bisherigen relativen Pfade zurueckkopieren und die im Manifest aufgefuehrten, vorher nicht vorhandenen Dateien entfernen. Erst nach vollstaendiger Wiederherstellung einen `rolled-back`-Marker in diesem Sicherungsordner anlegen oder die gepruefte Sicherung aus dem Sicherungsverzeichnis verschieben. Es werden keine Server- oder Datenbankupdates ausgefuehrt.

## Selbst bauen

Voraussetzungen: Rust stable mit MSVC-Toolchain und Visual Studio C++ Build Tools inkl. Windows SDK.

```powershell
cargo test
cargo build --release
```

Ergebnis: `target\x86_64-pc-windows-msvc\release\schild3-updater.exe`. Die C-Runtime wird statisch gelinkt. Standard-Windows-Systembibliotheken und Grafiktreiber werden weiterhin verwendet. Die EXE ist nicht digital signiert; Windows SmartScreen kann deshalb eine Warnung anzeigen.

Nur lesende Pruefung mit der Debug-EXE in einer Konsole:

```powershell
cargo run -- --check https://schule.example.org/ C:\SchILD3
```

Automatisierte Tests decken Versionszuordnung, Ablehnung unsicherer URLs und Archivpfade, Dateierhalt, Sicherung, Ruecksetzung und Windows-Dateisperren ab. Eine reale Schulinstallation wird durch die Tests nicht veraendert.

Ein optionaler Netzwerktest verwendet ein lokal heruntergeladenes aelteres offizielles ZIP als Ausgangsversion. Er aktualisiert ausschliesslich einen temporaeren Ordner, prueft den SHA-256-gesicherten Download, die neue EXE-Version, die Sicherung und den Erhalt lokaler Dateien:

```powershell
$env:SCHILD3_TEST_ARCHIVE = 'C:\Downloads\Setup_SchILD3_v3.5.0.zip'
$env:SCHILD3_TEST_SERVER = 'https://schule.example.org/'
cargo test official_release_end_to_end -- --ignored --nocapture
```

## Lizenz

Der eigene Projektcode steht unter der **[MIT-Lizenz](LICENSE)** (SPDX: `MIT`), einer liberalen, OSI-anerkannten Open-Source-Lizenz.

- Private und kommerzielle Nutzung sind erlaubt.
- Aenderung, Weitergabe und Verkauf sind erlaubt, auch als Teil proprietaerer Software.
- Der Urheberrechtsvermerk und der Lizenztext muessen bei Kopien oder wesentlichen Teilen der Software erhalten bleiben.
- Es besteht keine Pflicht, eigene Aenderungen zu veroeffentlichen.
- Die Software wird im gesetzlich zulaessigen Umfang ohne Gewaehrleistung bereitgestellt.

Massgeblich ist der englische Lizenztext. Die Projektlizenz aendert nicht die Lizenzen der eingebundenen Rust-Abhaengigkeiten oder der Lucide-/Feather-Icons. Grafikquellen stehen in [assets/README.md](assets/README.md), der mitgelieferte Icon-Lizenztext in [assets/lucide-LICENSE](assets/lucide-LICENSE). Marken- und Kennzeichenrechte an Namen und Firmenlogos werden durch die MIT-Lizenz nicht eingeraeumt.

## Mitwirken und Fehler melden

Fehlerberichte und Verbesserungsvorschlaege sind ueber [GitHub Issues](https://github.com/cschmidtoptima/Schild3Updater/issues) willkommen. Bitte Windows-Version, Updater-Version, die relevanten Versionsnummern und eine nachvollziehbare Fehlerbeschreibung angeben. Protokolle vor dem Hochladen auf Schulnamen, interne Serveradressen, UNC-Pfade und personenbezogene Daten pruefen und diese Angaben entfernen. Keine Schuelerdaten, Zugangsdaten oder kompletten Clientinstallationen hochladen.

Fuer Codebeitraege bitte einen Pull Request erstellen, bestehende Tests beruecksichtigen und `cargo fmt --check` sowie `cargo test --locked` ausfuehren. Bitte nur Code einreichen, fuer den Sie die erforderlichen Rechte besitzen und den Sie unter der MIT-Lizenz bereitstellen duerfen.

## Veroeffentlichung vorbereiten

- Repository-Beschreibung fuer das GitHub-About-Feld: **Windows-Updater fuer SchILD3 mit SVWS-Kompatibilitaetspruefung, Sicherungen und GitHub-Selbstaktualisierung. Open Source unter MIT.**
- Repository und Releases oeffentlich erreichbar machen und GitHub Actions aktivieren. Vorher Rechte am eigenen Code, Firmenlogo und sonstigen eingebundenen Inhalten pruefen.
- Lokale Einstellungen, Buildordner und EXE-Dateien gehoeren nicht in den Quellcode-Commit. Die Ignore-Regeln gelten nicht rueckwirkend fuer bereits versionierte Dateien; diese muessen aus dem Git-Index entfernt werden. Lokale Dateien koennen dabei erhalten bleiben.
- Vor einer oeffentlichen Freigabe auch die Git-Historie auf interne Informationen pruefen: Das Entfernen aus dem aktuellen Stand entfernt fruehere Versionen nicht. Eine eventuell erforderliche Historienbereinigung muss gesondert geplant werden.
- Lizenz und Hinweise zu Abhaengigkeiten bei der Weitergabe beilegen. Der Release-Workflow stellt die Projektlizenz, README und Icon-Lizenz neben der EXE bereit. Fuer die Rust-Abhaengigkeiten gelten zusaetzlich deren eigene Lizenz- und Hinweispflichten; vor einer Binaerveroeffentlichung pruefen und erforderliche Hinweise beilegen.
- Aenderungen committen und pushen; danach wie oben beschrieben einen zur Cargo-Version passenden Tag veroeffentlichen. Es werden keine Schulkonfigurationen in den Release aufgenommen.