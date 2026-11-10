# SchILD3 Updater

Windows-Desktopprogramm in Rust. Die fertige EXE benoetigt weder eine Rust-Installation noch eine zusaetzliche WebView-/VC-Runtime. Unterstuetzt wird Windows x64.

Die Oberflaeche verwendet das eingebettete OPTIMA-Logo und einen anklickbaren Verweis auf <https://optima-computer.de/>. Das eigenstaendige Programm-Icon zeigt ein Dokument mit der Ziffer 3 und Aktualisierungspfeilen, nicht das Firmenlogo. Logo und Icons funktionieren auch offline. Groessere Schrift, skalierbare Bedienicons und Per-Monitor-DPI-Unterstuetzung verbessern die Darstellung auf hochaufloesenden Bildschirmen; soweit vorhanden wird die installierte Windows-Schrift Segoe UI verwendet. Unter **Info** stehen die Programmversion und die Lizenzhinweise der Bedienicons.

## Bedienung

1. `schild3-updater.exe` starten.
2. HTTPS-Adresse der SVWS-Instanz eintragen.
3. Vorhandenen SchILD3-Clientordner mit `SchILD30.exe` waehlen oder einen UNC-Pfad wie `\\server\freigabe\SchILD3` eingeben.
4. Einstellungen ueber das Speichersymbol rechts neben **Verbindung und Client** speichern. Die Datei `settings.json` wird direkt neben der Updater-EXE angelegt und von dort geladen, unabhaengig vom Arbeitsverzeichnis. Die Einstellungen werden auch beim Start einer Pruefung gespeichert.
5. SchILD3 vor einem Update auf allen betroffenen Arbeitsplaetzen schliessen.
6. **Nur pruefen** zeigt die Versionen an. **Pruefen und aktualisieren** installiert bei Bedarf automatisch den passenden neueren Client.

Bei jedem Programmstart wird mit den gespeicherten Einstellungen einmal automatisch im Hintergrund auf eine passende neuere Clientversion geprueft. Ohne konfigurierte Serveradresse und Clientordner wird keine Pruefung gestartet. Fehler erscheinen im Hauptfenster und im Protokoll.

Ist ein Update verfuegbar, erscheint ein Dialog mit Serverversion sowie installierter und angebotener Clientversion. **OK** startet den Download und die Installation dieser Version. **Abbrechen**, Escape oder das Schliessen des Dialogs verwirft das Angebot, ohne Clientdateien zu veraendern. Bei bereits passendem Client erscheint kein Update-Dialog. Vor der Installation werden Server- und Clientversion erneut geprueft.

Das Programm fuehrt keine unbeaufsichtigten Hintergrundupdates aus und startet SchILD3 nicht automatisch. Bei UNC-Pfaden gelten die Netzwerkberechtigungen des angemeldeten Windows-Benutzers. Zugangsdaten werden nicht gespeichert. Schreibrechte im Updater-Ordner zum Speichern der Einstellungen und im gesamten betroffenen Clientordner zum Aktualisieren sind erforderlich. Eine bisher unter `%APPDATA%\SchILD3Updater\settings.json` gespeicherte Konfiguration wird nicht automatisch uebernommen; sie kann bei Bedarf neben die Updater-EXE verschoben werden.

## Versionsauswahl

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