app-title = Actualizador
app-description = Configura, programa y ejecuta actualizaciones de todo el sistema con topgrade.

## Navigation

nav-overview = Resumen
nav-schedule = Programación
nav-configuration = Configuración
nav-run = Ejecutar

category-system = Sistema
category-applications = Aplicaciones
category-containers = Contenedores
category-development = Desarrollo
category-editors = Editores
category-repositories = Repositorios
category-shell = Intérprete de órdenes
category-ai-tools = Herramientas de IA
category-cloud = Nube
category-desktop = Escritorio
category-custom = Órdenes personalizadas
category-other = Otros

## Overview

overview-heading = Fuentes de actualización
overview-subtitle = { $available } de { $total } pasos se aplican a este sistema.
topgrade-version = topgrade { $version }
topgrade-source-system = Instalado en este sistema
topgrade-source-bundled = Incluido con esta aplicación
scanning = Comprobando qué se aplica a este sistema…
scanning-progress = Comprobados { $completed } de { $total } — { $step }
rescan = Volver a comprobar
rescan-tooltip = Buscar de nuevo herramientas recién instaladas

## Steps

steps-heading = Pasos
steps-none = No hay pasos en esta categoría.
step-available = Listo
step-unavailable = No disponible
step-inactive = No aplicable
step-deprecated = Obsoleto
step-enabled-tooltip = Incluir este paso al actualizar
step-disabled-tooltip = Omitir este paso al actualizar
step-components = { $count ->
    [one] 1 componente
   *[other] { $count } componentes
}
enable-all = Activar todo
disable-all = Desactivar todo
show-unavailable = Mostrar pasos no disponibles
show-unavailable-tooltip = Listar también los pasos cuyas herramientas no están instaladas

status-ok = Listo
status-skipped = Omitido
status-failed = Fallido

## Running

run-heading = Ejecutar
run-now = Actualizar ahora
dry-run = Vista previa
dry-run-tooltip = Mostrar qué se haría sin cambiar nada
run-in-progress = Actualizando…
run-step = { $step }
run-finished = Finalizado
run-cancelled = Cancelado
run-failed = Finalizado con errores
run-never = Todavía no se ha ejecutado ninguna actualización.
run-last = Última ejecución { $when }
run-summary = { $ok } correctos, { $skipped } omitidos, { $failed } fallidos
cancel-run = Detener
clear-log = Limpiar
copy-log = Copiar salida
run-selected-only = Responder afirmativamente a los gestores de paquetes

## Authentication

password-title = Se requiere la contraseña de administrador
password-body = { $command } necesita permisos de administrador para continuar.
password-placeholder = Contraseña
authenticate = Autenticar
authentication-failed = Esa contraseña no se aceptó.

## Schedule

schedule-heading = Actualizaciones programadas
schedule-enabled = Buscar actualizaciones de forma programada
schedule-frequency = Frecuencia
frequency-hourly = Cada hora
frequency-daily = Diariamente
frequency-weekly = Semanalmente
frequency-monthly = Mensualmente
schedule-time = Hora del día
schedule-automatic = Instalar actualizaciones automáticamente
schedule-automatic-description = Si está desactivado, una notificación informa de lo disponible y no se cambia nada.
schedule-next-run = Próxima ejecución { $when }
schedule-next-run-unknown = No se conoce la hora de la próxima ejecución.
schedule-backend-systemd = Se ejecuta en segundo plano mediante un temporizador de usuario de systemd, incluso con esta ventana cerrada.
schedule-backend-fallback = systemd no está disponible, así que las ejecuciones programadas solo ocurren mientras esta ventana está abierta.
schedule-apply = Aplicar programación
schedule-applied = Programación actualizada.
schedule-error = No se pudo aplicar la programación: { $message }

## Configuration

configuration-heading = Configuración de topgrade
configuration-path = Editando { $path }
configuration-default = Predeterminado: { $value }
configuration-not-set = Sin definir
configuration-save = Guardar cambios
configuration-revert = Descartar
configuration-reset = Restablecer al valor predeterminado
configuration-unsaved = Hay cambios sin guardar.
configuration-saved = Configuración guardada.
configuration-free-form = Son órdenes que usted mismo nombra. Edite esta sección directamente en el archivo.
configuration-open-file = Abrir el archivo de configuración
configuration-add = Añadir
configuration-remove = Quitar

## Application settings

settings = Ajustes
about = Acerca de
appearance = Apariencia
theme = Tema
theme-system = Como el escritorio
theme-light = Claro
theme-dark = Oscuro
behaviour = Comportamiento
privilege-backend = Permisos de administrador
privilege-pty = Preguntar en esta ventana
privilege-pty-description = Ejecuta topgrade en un terminal y pregunta aquí cuando hace falta una contraseña.
privilege-pkexec = Diálogo del sistema
privilege-pkexec-description = Usa el diálogo de autenticación del escritorio. Pregunta una vez por orden.
confirm-before-running = Confirmar antes de iniciar una actualización
notify-on-completion = Notificar cuando termine una ejecución programada

## Errors and empty states

topgrade-missing-title = topgrade no está instalado
topgrade-missing-body = Esta aplicación controla topgrade, que no se encontró en este sistema.
topgrade-missing-hint = Instálelo con su gestor de paquetes o con: { $command }
topgrade-too-old-title = topgrade es demasiado antiguo
topgrade-too-old-body = Se encontró topgrade { $found }, pero se necesita { $required } o posterior.
error-title = Algo salió mal
retry = Reintentar

## Common

cancel = Cancelar
close = Cerrar
save = Guardar
ok = Aceptar
toggle-sidebar = Alternar la barra lateral
git-description = Descripción de Git
repository = Repositorio
support = Soporte

## History, first run, custom commands and status area

autostart = Iniciar con la sesión
autostart-description = Añade una entrada en ~/.config/autostart
category-settings = Ajustes de esta categoría
category-settings-none = Esta categoría no tiene ajustes propios de topgrade.
command-name-placeholder = Nombre
command-value-placeholder = Orden a ejecutar
custom-commands-description = Órdenes que usted mismo nombra. topgrade las ejecuta como un paso propio.
custom-commands-none = Todavía no hay órdenes personalizadas.
first-run-accept = Continuar
first-run-autostart = Iniciar con la sesión
first-run-autostart-description = Se inicia minimizado al entrar para que las comprobaciones programadas puedan ejecutarse.
first-run-body = Estas opciones cambian cómo se comporta la aplicación fuera de su propia ventana. Puede cambiarlas más tarde en Ajustes.
first-run-title = Un par de decisiones
first-run-tray = Mostrar un icono en el área de estado
first-run-tray-description = Permite ocultar la ventana y recuperarla, e iniciar una actualización sin abrirla.
history-back = Volver a la lista
history-delete = Eliminar
history-detail = { $outcome } · { $origin } · { $duration }
history-duration-seconds = { $seconds } s
history-heading = Ejecuciones anteriores
history-none = Todavía no se ha registrado ninguna ejecución.
history-origin-manual = Iniciada aquí
history-origin-scheduled = Programada
history-outcome-cancelled = Cancelada
history-outcome-failed = Fallida
history-outcome-succeeded = Correcta
history-transcript-unavailable = No se pudo leer el registro de esta ejecución.
view = Ver
keep-run-logs = Ejecuciones que conservar
nav-history = Historial
notify-failed-steps = Fallaron: { $steps }
notify-title-failed = Actualización finalizada con errores
notify-title-succeeded = Actualización finalizada
show-tray-icon = Mostrar un icono en el área de estado
tray-quit = Salir
tray-show = Mostrar la ventana
tray-unavailable = No se encontró un área de estado en este escritorio, así que no se muestra ningún icono.

## Releases

nav-releases = Publicaciones
releases-add-selected = Vigilar la selección
releases-cancel-find = Cancelar
releases-check = Buscar actualizaciones
releases-checking = { $done } de { $total } comprobados…
releases-description = El software instalado desde la página de publicaciones de un proyecto no tiene un gestor de paquetes detrás, así que topgrade no puede actualizarlo. Estos se comprueban en el propio proyecto.
releases-error = No se pudo comprobar: { $message }
releases-find = Buscar proyectos
releases-finding = Revisando los paquetes instalados…
releases-found = Se encontraron { $count } proyectos en este sistema. Elija cuáles vigilar.
releases-heading = Publicaciones de proyectos
releases-installed = { $name } actualizado a { $version }
releases-install-failed = No se pudo actualizar { $name }: { $message }
releases-installing = Instalando { $name }…
releases-no-asset = Ningún archivo de esta publicación corresponde a este sistema; use la página de publicación.
releases-none = Todavía no se vigila ningún proyecto.
releases-no-releases = Sin publicaciones
releases-no-transport = No están instalados ni curl ni gh, así que no se pueden comprobar las publicaciones.
releases-open = Página de la publicación
releases-remove = Dejar de vigilar
releases-source = { $source } · { $forge }
releases-unidentified = Proyecto desconocido — { $version } instalado
releases-unknown = { $version } publicada
releases-update = Actualizar
releases-update-available = { $version } disponible
releases-up-to-date = Al día
releases-watched = { $count } vigilados
run-was-preview = Esto fue una vista previa: no se cambió nada en el sistema.

interval-daily = Diariamente
interval-manual = Solo cuando se pida
interval-six-hourly = Cada 6 horas
interval-weekly = Semanalmente
releases-interval = Comprobar automáticamente
releases-last-checked = Comprobado por última vez { $when }
releases-never-checked = Aún sin comprobar
releases-next-check = siguiente { $when }

## Dependencies, release channel and directories

channel-pre-release = Incluir betas y candidatas a versión final
channel-stable = Solo estables
dep-authentication-dismissed = Se canceló la autenticación.
dep-curl = Obtiene información de publicaciones de los servidores de proyectos y descarga actualizaciones.
dependencies-all-present = Todo lo que esta aplicación necesita está instalado.
dependencies-description = Esta aplicación funciona controlando otros programas. Uno que falte se convierte en una función que no hace nada en silencio, así que aquí se listan con su finalidad.
dependencies-heading = Herramientas necesarias
dependencies-install = Instalar
dependencies-installed = Instalado
dependencies-install-failed = No se pudo instalar { $name }: { $message }
dependencies-installing = Instalando…
dependencies-missing = No instalado
dependencies-no-manager = No se encontró un gestor de paquetes compatible, así que no se puede instalar desde aquí.
dependencies-optional = Opcional
dependencies-recheck = Comprobar de nuevo
dependencies-required = Necesario
dep-gh = Aporta sus credenciales de GitHub, elevando el límite de comprobaciones de 60 a 5000 peticiones por hora.
dep-notify-send = Informa del resultado de una ejecución programada que nadie estaba mirando.
dep-pkexec = Pide permisos de administrador mediante el diálogo del escritorio, para actualizaciones del sistema e instalación de paquetes.
dep-systemctl = Mantiene la programación como un temporizador de usuario de systemd, para que se ejecute con esta ventana cerrada.
dep-topgrade = Realiza las actualizaciones. Sin él, esta aplicación no tiene nada que controlar.
dep-xdg-open = Abre páginas de publicaciones y enlaces en su navegador.
nav-dependencies = Dependencias
releases-channel = Publicaciones a ofrecer
releases-directories = Carpetas de aplicaciones descargadas
releases-directories-description = Se buscan AppImages y otros programas descargados. Las rutas relativas parten de su carpeta personal.
releases-directory-add = Añadir carpeta
releases-directory-placeholder = Applications
releases-self = Esta aplicación
dependencies-missing-required = { $count ->
    [one] Falta 1 herramienta necesaria.
   *[other] Faltan { $count } herramientas necesarias.
}
releases-channel-description = Si las candidatas y las betas cuentan como actualizaciones.

## Welcome, notifications and virus scanning

clamav-clean = Análisis terminado: { $scanned } archivos comprobados, no se encontró nada.
clamav-failed = No se pudo ejecutar el análisis: { $message }
clamav-infected = Análisis terminado: { $infected } archivo(s) infectado(s).
clamav-options = Opciones de análisis
clamav-scan = Analizar tras actualizarse la base de datos de virus
clamav-scan-description = ClamAV está instalado. topgrade mantiene su base de datos al día; esto analiza con la nueva base en cuanto cambia.
clamav-scanning = La base de datos de virus cambió: analizando…
clamav-target = Qué analizar
nav-welcome = Bienvenida
notify-errors = Avisarme cuando una actualización falle
notify-errors-description = Los fallos se avisan aunque el resto de notificaciones estén desactivadas, salvo que esto también se desactive.
notify-title-available = Hay actualizaciones disponibles
notify-title-installed = Actualizaciones instaladas
notify-upgrades = Avisarme de las actualizaciones
notify-upgrades-available = Se le dirá qué hay disponible para instalar.
notify-upgrades-installed = Se le dirá qué se ha instalado.
welcome-automatic-heading = Instalación de actualizaciones
welcome-body = Unas pocas decisiones que conviene tomar ahora. Todas están después en Ajustes, y nada de esto es permanente.
welcome-clamav = Análisis de virus
welcome-finish = Listo
welcome-heading = Configurar las actualizaciones
welcome-notifications = Notificaciones
welcome-root-warning = La instalación desatendida necesita permisos de administrador, así que la ejecución programada se instala como un servicio del sistema que corre como root. Nada más en esta aplicación se ejecuta como root.

## Package sources

nav-sources = Fuentes de paquetes
sources-add-apt = Añadir fuente APT
sources-add-flatpak = Añadir remoto Flatpak
sources-add-heading = Añadir una fuente
sources-apt-hint = Una fuente APT se escribe en /etc/apt/sources.list.d y requiere permisos de administrador.
sources-changing = Aplicando…
sources-description = Los repositorios de los que tiran sus gestores de paquetes. topgrade actualiza lo instalado; estos deciden qué hay disponible.
sources-disable-note = Las fuentes de APT y dnf se desactivan en vez de borrarse, para poder deshacer el cambio a mano.
sources-disabled = Desactivada
sources-enabled = Activada
sources-flatpak-hint = Un remoto Flatpak se añade solo para usted y no necesita contraseña. Indique una URL .flatpakrepo.
sources-heading = De dónde vienen los paquetes
sources-name-placeholder = Nombre
sources-none = No se encontraron fuentes de paquetes.
sources-privileged = Cambiar esto requiere permisos de administrador.
sources-reload = Recargar
sources-remove = Quitar
sources-suite-placeholder = Serie (p. ej. stable)
sources-url-placeholder = URL
show-tray-icon-description = Trae la ventana al frente, inicia una actualización sin abrirla y sale. No puede ocultar la ventana: Wayland no ofrece forma de deshacerlo.
welcome-show-again = Pantalla de configuración
welcome-show-again-description = Volver a la primera pantalla y sus opciones.

releases-restart-failed = No se pudo iniciar la nueva versión: { $message }
releases-restart-unknown-path = no se conoce la ubicación de esta aplicación
releases-restarting = Actualizado: reiniciando en la nueva versión…
