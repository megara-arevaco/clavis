# Revisión de implementación — bloque local no comercial

Alcance: completar mejoras locales y verificables de Clavis, sin alterar instancias del usuario, teclados, permisos, compositor ni configuración del sistema. No se hizo despliegue, publicación, telemetría, monetización ni commit. Este cierre cubre este bloque; no declara terminadas propuestas externas a él.

## Resolución

| Área | Estado y evidencia | Límite / trabajo externo |
|---|---|---|
| Atajo del compositor como alternativa a ACL | README e Info recomiendan `clavis --show` mediante un atajo del compositor en Wayland. Se conserva doble Ctrl opcional. | No se cambió ni probó la configuración de GNOME u otro compositor. |
| Diagnóstico pasivo | Info muestra dispositivo configurado, nombre/ruta observables en sysfs y un posible evento renumerado por nombre. Producción consulta configuración de Clavis y metadatos sysfs; no abre `/dev/input`, inspecciona ACL ni lee eventos. La reconexión se prueba con datos sintéticos. | El nombre puede no distinguir dos teclados idénticos. Desconexión, permisos y reconexión físicos no se probaron; no se ejecutaron `--check-input` ni `--diagnose`. |
| Búsqueda y accesos | Alcance de búsqueda actual/todas las herramientas, favoritos, etiquetas de tipo, controles localizados, escala normal/grande y distribución responsive de una/dos columnas. Pruebas unitarias y smoke GTK aislado. | No hubo prueba de usabilidad observada ni de DPI/ventanas en un compositor real. |
| Referencias personales | UI para crear/editar; importación JSON schema 1 y exportación, también vacía; validación estricta, HTTPS, límites, duplicados rechazados y sin reemplazar referencias existentes. Importar agrega; editar cambia solo la selección explícita. | El smoke recorre widgets y funciones de importación/exportación, pero no abre ni confirma los diálogos nativos de archivos manualmente. |
| Persistencia | Archivos limitados a 1 MiB; escritura atómica, archivos nuevos con modo `0600`, rutas con symlinks rechazadas y exportación que no reemplaza un destino. Hay pruebas de symlink, límite, fallo y no sobrescritura. | Los archivos no se cifran. Las comprobaciones de ruta no son una defensa frente a un proceso local hostil que cambie directorios concurrentemente durante una operación. |
| Revisión de riesgo del catálogo | `research/catalog-risk-review.json` clasifica individualmente las 94 entradas de `catalog.json` y 15 de Ghostty: 27 advertencias exactas y 82 decisiones explícitas sin advertencia adicional. Una prueba comprueba cobertura uno-a-uno, texto exacto, ausencia de alertas no registradas y fingerprints. | La clasificación es conservadora, no interpreta comandos ni demuestra que una entrada sin aviso sea segura. Referencias personales solo pueden traer avisos escritos por su autor; Clavis nunca ejecuta el catálogo. |
| Procedencia / versiones | `research/RESEARCH.md` y `research/sources.json` registran los snapshots archivados. La UI indica origen y versión/revisión cuando hay evidencia; ausencias se muestran como `Not recorded`. Ghostty distingue configuración explícita de defaults y no resuelve includes/condiciones. | No se compararon snapshots con versiones instaladas o upstream actuales. La fecha de revisión local permanece `Not recorded`; no se infiere vigencia. |
| Accesibilidad visual | Nombres accesibles explícitos en controles clave; Info visible, atajos/foco y botones comprensibles. Capturas propias de la app en inglés/español, normal/grande y una columna. | El smoke inspecciona el árbol GTK, no es una prueba con lector de pantalla ni una auditoría AT-SPI completa. |
| Monetización / servicios | Sin pagos, donaciones, servidor, telemetría ni prestaciones de pago. | No forma parte del alcance delegado. |

## GTK aislado y evidencia visual

`--test-mode` y `--capture` usan un ID de aplicación separado, desactivan listener/tray y muestran diagnóstico sintético. Ambos requieren `CLAVIS_TEST_CONFIG_HOME` como ruta hija sin symlinks del directorio temporal; Clavis la usa como `XDG_CONFIG_HOME`, evitando preferencias del usuario. El smoke también evita consultar sysfs o leer estado de dispositivos. Ctrl+F/Esc se ejercitan mediante el manejador GTK sintético, no enviando teclas físicas.

Capturas generadas con `--capture --width 430 --tool Git` en D-Bus privado y XDG temporal; son PNG solo de Clavis:

- `docs/screenshots/validation/clavis-en-normal-one-column.png` — 430×440.
- `docs/screenshots/validation/clavis-en-large-one-column.png` — 430×440.
- `docs/screenshots/validation/clavis-es-normal-one-column.png` — 434×440.
- `docs/screenshots/validation/clavis-es-large-one-column.png` — 466×440.

La anchura algo mayor en español refleja el mínimo de sus controles traducidos; las cuatro capturas mantienen una columna. La descripción de las referencias del catálogo sigue en inglés en algunas filas españolas. `docs/screenshots/clavis.png` era un cambio local previo y se preservó; no es una de estas cuatro capturas.

## Validación ejecutada

- `cargo test --locked`: **26 passed, 0 failed**.
- `cargo build --release --locked`: compilación release completada; `./target/release/clavis --version` imprimió `clavis 0.4.3`.
- `cargo build --locked`: compilación debug completada.
- Smoke GTK final, `CLAVIS_TEST_CONFIG_HOME=<tmp>/config timeout 20s dbus-run-session -- env GTK_USE_PORTAL=0 GDK_BACKEND=x11 ./target/debug/clavis --test-mode`: **17 passed, 0 failed**. Incluye búsqueda, favoritos/persistencia temporal, idioma/escala, Info/refresco sintético, manejadores Ctrl+F/Esc y editor/importación/exportación personal. Al ejecutar el modo sin `CLAVIS_TEST_CONFIG_HOME`, salió **1** con `Isolated GTK mode refused: set CLAVIS_TEST_CONFIG_HOME to a child of the system temporary directory`.
- Capturas `--capture` terminadas con tamaños listados arriba. Los portales del D-Bus privado emitieron avisos de canal Wayland/GVFS/secretos al arrancar/cerrar; no impidieron ni forman parte de las 17 aserciones GTK.
- `git diff --check`: sin errores.
- `cargo fmt -- --check` no disponible: el entorno no tiene el subcomando `fmt`/`rustfmt`.

No se abrieron teclados, no se capturaron entradas reales, no se concedieron ACLs, no se cambiaron grupos, no se probaron tray/compositor físicos y no se ejecutó ningún comando del catálogo. Tampoco se cerraron procesos del usuario.

## Decisiones y pendientes para coordinación

1. Mantener visibles los límites: las advertencias no son análisis de shell ni garantía de seguridad; `no_warning` no significa “seguro”.
2. La fecha de revisión y algunas versiones upstream siguen sin evidencia archivada; no afirmar que el catálogo está actualizado. Revisar versiones en otro bloque si se solicita y existe un método seguro.
3. Quedan validaciones físicas separadas: lector de pantalla/AT-SPI, selector de archivos nativo, atajo del compositor, tray, foco/posición, y desconexión/reconexión/permisos con dispositivos reales. No se simularon como aprobadas.
4. Conservar los cambios locales preexistentes en `src/style.css` y `docs/screenshots/clavis.png`; no se tocaron en este bloque. No hubo commit.
