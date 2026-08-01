app-title = Atualizador
app-description = Configure, agende e execute atualizações de todo o sistema com o topgrade.

## Navigation

nav-overview = Visão geral
nav-schedule = Agendamento
nav-configuration = Configuração
nav-run = Executar

category-system = Sistema
category-applications = Aplicativos
category-containers = Contêineres
category-development = Desenvolvimento
category-editors = Editores
category-repositories = Repositórios
category-shell = Shell
category-ai-tools = Ferramentas de IA
category-cloud = Nuvem
category-desktop = Área de trabalho
category-custom = Comandos personalizados
category-other = Outros

## Overview

overview-heading = Fontes de atualização
overview-subtitle = { $available } de { $total } etapas se aplicam a este sistema.
topgrade-version = topgrade { $version }
topgrade-source-system = Instalado neste sistema
topgrade-source-bundled = Incluído com este aplicativo
scanning = Verificando o que se aplica a este sistema…
scanning-progress = { $completed } de { $total } verificadas — { $step }
rescan = Verificar novamente
rescan-tooltip = Procurar novamente por ferramentas recém-instaladas

## Steps

steps-heading = Etapas
steps-none = Nenhuma etapa nesta categoria.
step-available = Pronta
step-unavailable = Indisponível
step-inactive = Não se aplica
step-deprecated = Obsoleta
step-enabled-tooltip = Incluir esta etapa ao atualizar
step-disabled-tooltip = Ignorar esta etapa ao atualizar
step-components = { $count ->
    [one] 1 componente
   *[other] { $count } componentes
}
enable-all = Ativar tudo
disable-all = Desativar tudo
show-unavailable = Mostrar etapas indisponíveis
show-unavailable-tooltip = Listar também as etapas cujas ferramentas não estão instaladas

status-ok = Pronta
status-skipped = Ignorada
status-failed = Falhou

## Running

run-heading = Executar
run-now = Iniciar atualização
dry-run = Prévia
dry-run-tooltip = Mostrar o que seria feito sem alterar nada
run-in-progress = Atualizando…
run-step = { $step }
run-finished = Concluído
run-cancelled = Cancelado
run-failed = Concluído com erros
run-never = Nenhuma atualização foi executada ainda.
run-last = Última execução { $when }
run-summary = { $ok } com êxito, { $skipped } ignoradas, { $failed } com falha
cancel-run = Parar
clear-log = Limpar
copy-log = Copiar a saída
run-selected-only = Responder afirmativamente aos gerenciadores de pacotes

## Authentication

password-title = Senha de administrador necessária
password-body = { $command } precisa de permissões de administrador para continuar.
password-placeholder = Senha
authenticate = Autenticar
authentication-failed = Essa senha não foi aceita.

## Schedule

schedule-heading = Atualizações agendadas
schedule-enabled = Procurar atualizações conforme um agendamento
schedule-frequency = Frequência
frequency-hourly = A cada hora
frequency-daily = Diariamente
frequency-weekly = Semanalmente
frequency-monthly = Mensalmente
schedule-time = Hora do dia
schedule-automatic = Instalar atualizações automaticamente
schedule-automatic-description = Quando desativado, uma notificação informa o que está disponível e nada é alterado.
schedule-next-run = Próxima execução { $when }
schedule-next-run-unknown = O horário da próxima execução não é conhecido.
schedule-backend-systemd = Executa em segundo plano por um temporizador de usuário do systemd, mesmo com esta janela fechada.
schedule-backend-fallback = O systemd não está disponível, portanto as execuções agendadas só ocorrem enquanto esta janela estiver aberta.
schedule-apply = Aplicar agendamento
schedule-applied = Agendamento atualizado.
schedule-error = Não foi possível aplicar o agendamento: { $message }

## Configuration

configuration-heading = Configuração do topgrade
configuration-path = Editando { $path }
configuration-default = Padrão: { $value }
configuration-not-set = Não definido
configuration-save = Salvar alterações
configuration-revert = Reverter
configuration-reset = Restaurar o padrão
configuration-unsaved = Há alterações não salvas.
configuration-saved = Configuração salva.
configuration-free-form = São comandos nomeados por você. Edite esta seção diretamente no arquivo.
configuration-open-file = Abrir o arquivo de configuração
configuration-add = Adicionar
configuration-remove = Remover

## Application settings

settings = Configurações
about = Sobre
appearance = Aparência
theme = Tema
theme-system = Como a área de trabalho
theme-light = Claro
theme-dark = Escuro
behaviour = Comportamento
privilege-backend = Permissões de administrador
privilege-pty = Perguntar nesta janela
privilege-pty-description = Executa o topgrade em um terminal e pergunta aqui quando uma senha for necessária.
privilege-pkexec = Diálogo do sistema
privilege-pkexec-description = Usa o diálogo de autenticação da área de trabalho. Pergunta uma vez por comando.
confirm-before-running = Confirmar antes de iniciar uma atualização
notify-on-completion = Notificar quando uma execução agendada terminar

## Errors and empty states

topgrade-missing-title = O topgrade não está instalado
topgrade-missing-body = Este aplicativo controla o topgrade, que não foi encontrado neste sistema.
topgrade-missing-hint = Instale-o com seu gerenciador de pacotes, ou com: { $command }
topgrade-too-old-title = O topgrade é antigo demais
topgrade-too-old-body = O topgrade { $found } foi encontrado, mas é necessário o { $required } ou mais recente.
error-title = Algo deu errado
retry = Tentar novamente

## Common

cancel = Cancelar
close = Fechar
save = Salvar
ok = OK
toggle-sidebar = Alternar a barra lateral
git-description = Descrição do Git
repository = Repositório
support = Suporte

## History, first run, custom commands and status area

autostart = Iniciar com a sessão
autostart-description = Adiciona uma entrada em ~/.config/autostart
category-settings = Configurações desta categoria
category-settings-none = Esta categoria não tem configurações próprias do topgrade.
command-name-placeholder = Nome
command-value-placeholder = Comando a executar
custom-commands-description = Comandos nomeados por você. O topgrade os executa como uma etapa própria.
custom-commands-none = Ainda não há comandos personalizados.
first-run-accept = Continuar
first-run-autostart = Iniciar com a sessão
first-run-autostart-description = Inicia minimizado ao entrar para que as verificações agendadas possam ser executadas.
first-run-body = Estas opções mudam como o aplicativo se comporta fora da sua própria janela. Você pode alterá-las depois nas Configurações.
first-run-title = Algumas escolhas
first-run-tray = Mostrar um ícone na área de status
first-run-tray-description = Permite ocultar a janela e trazê-la de volta, e iniciar uma atualização sem abri-la.
hide-to-tray = Ocultar na área de status
history-back = Voltar à lista
history-delete = Excluir
history-detail = { $outcome } · { $origin } · { $duration }
history-duration-seconds = { $seconds } s
history-heading = Execuções anteriores
history-none = Nenhuma execução foi registrada ainda.
history-origin-manual = Iniciada aqui
history-origin-scheduled = Agendada
history-outcome-cancelled = Cancelada
history-outcome-failed = Falhou
history-outcome-succeeded = Bem-sucedida
history-transcript-unavailable = Não foi possível ler o registro desta execução.
view = Ver
keep-run-logs = Execuções a manter
minimize-to-tray = Ocultar na área de status em vez de sair
minimize-to-tray-description = Adiciona um botão Ocultar. O botão de fechar da janela ainda sai.
nav-history = Histórico
notify-failed-steps = Falharam: { $steps }
notify-title-failed = Atualização concluída com erros
notify-title-succeeded = Atualização concluída
show-tray-icon = Mostrar um ícone na área de status
tray-hide = Ocultar a janela
tray-quit = Sair
tray-show = Mostrar a janela
tray-unavailable = Nenhuma área de status foi encontrada nesta área de trabalho, portanto nenhum ícone é mostrado.

## Releases

nav-releases = Lançamentos
releases-add-selected = Acompanhar a seleção
releases-cancel-find = Cancelar
releases-check = Procurar atualizações
releases-checking = { $done } de { $total } verificados…
releases-description = Softwares instalados pela página de lançamentos de um projeto não têm um gerenciador de pacotes por trás, então o topgrade não pode atualizá-los. Estes são verificados no próprio projeto.
releases-error = Não foi possível verificar: { $message }
releases-find = Procurar projetos
releases-finding = Examinando os pacotes instalados…
releases-found = { $count } projetos encontrados neste sistema. Escolha quais acompanhar.
releases-heading = Lançamentos de projetos
releases-installed = { $name } atualizado para { $version }
releases-install-failed = Não foi possível atualizar { $name }: { $message }
releases-installing = Instalando { $name }…
releases-no-asset = Nenhum arquivo deste lançamento corresponde a este sistema; use a página do lançamento.
releases-none = Nenhum projeto está sendo acompanhado ainda.
releases-no-releases = Nenhum lançamento
releases-no-transport = Nem curl nem gh está instalado, portanto os lançamentos não podem ser verificados.
releases-open = Página do lançamento
releases-remove = Parar de acompanhar
releases-source = { $source } · { $forge }
releases-unidentified = Projeto desconhecido — { $version } instalado
releases-unknown = { $version } publicado
releases-update = Atualizar
releases-update-available = { $version } disponível
releases-up-to-date = Atualizado
releases-watched = { $count } acompanhados
run-was-preview = Isto foi uma prévia — nada no sistema foi alterado.

interval-daily = Diariamente
interval-manual = Apenas quando solicitado
interval-six-hourly = A cada 6 horas
interval-weekly = Semanalmente
releases-interval = Verificar automaticamente
releases-last-checked = Verificado pela última vez { $when }
releases-never-checked = Ainda não verificado
releases-next-check = próxima { $when }

## Dependencies, release channel and directories

channel-pre-release = Incluir betas e candidatas a lançamento
channel-stable = Apenas estáveis
dep-authentication-dismissed = A autenticação foi cancelada.
dep-curl = Obtém informações de lançamentos dos servidores dos projetos e baixa atualizações.
dependencies-all-present = Tudo o que este aplicativo precisa está instalado.
dependencies-description = Este aplicativo funciona controlando outros programas. Um que falte vira um recurso que silenciosamente não faz nada, então eles estão listados aqui com sua finalidade.
dependencies-heading = Ferramentas necessárias
dependencies-install = Instalar
dependencies-installed = Instalado
dependencies-install-failed = Não foi possível instalar { $name }: { $message }
dependencies-installing = Instalando…
dependencies-missing = Não instalado
dependencies-no-manager = Nenhum gerenciador de pacotes compatível foi encontrado, então nada pode ser instalado daqui.
dependencies-optional = Opcional
dependencies-recheck = Verificar de novo
dependencies-required = Necessário
dep-gh = Usa suas credenciais do GitHub, elevando o limite de verificações de 60 para 5000 requisições por hora.
dep-notify-send = Informa o resultado de uma execução agendada que ninguém estava acompanhando.
dep-pkexec = Pede permissões de administrador pelo diálogo da área de trabalho, para atualizações do sistema e instalação de pacotes.
dep-systemctl = Mantém o agendamento como um temporizador de usuário do systemd, para rodar com esta janela fechada.
dep-topgrade = Realiza as atualizações em si. Sem ele, este aplicativo não tem o que controlar.
dep-xdg-open = Abre páginas de lançamentos e links no seu navegador.
nav-dependencies = Dependências
releases-channel = Lançamentos a oferecer
releases-directories = Pastas de aplicativos baixados
releases-directories-description = São procurados AppImages e outros programas baixados. Caminhos relativos partem da sua pasta pessoal.
releases-directory-add = Adicionar pasta
releases-directory-placeholder = Applications
releases-self = Este aplicativo
dependencies-missing-required = { $count ->
    [one] Falta 1 ferramenta necessária.
   *[other] Faltam { $count } ferramentas necessárias.
}
releases-channel-description = Se candidatas a lançamento e betas contam como atualizações.
