# Settupper

> **Gerenciador de pacotes declarativo com TUI — configure uma vez, rode em qualquer máquina.**

Settupper é uma aplicação de terminal (TUI) que lê um arquivo `YAML` ou `JSON` com a lista de programas que você precisa, verifica o que já está instalado e executa install, update ou uninstall com um clique — sem lembrar de comandos específicos de cada distro ou sistema operacional.

![Preview](https://github.com/devbaraus/settupper/blob/main/assets/image.png)

---

## Por que usar?

- Você formata o computador com frequência e cansa de reinstalar tudo na mão
- Sua equipe precisa de um ambiente padronizado sem scripts frágeis de shell
- Você mantém múltiplas máquinas (Linux pessoal, Mac do trabalho, VM Windows) e quer uma config única
- Prefere declarar *o que quer* em vez de lembrar *como instalar* em cada OS

---

## Funcionalidades

- **TUI interativa** com painel de lista, detalhes e log em tempo real
- **Multi-plataforma**: Ubuntu, Fedora (e derivados), macOS, Windows
- **Ações**: install, update, uninstall, smart (decide automaticamente)
- **Smart All**: instala/atualiza tudo de uma vez respeitando dependências entre apps
- **Dependências entre apps** com ordenação topológica — se `nvm` depende de `git`, git é instalado primeiro
- **Reboot flag** — se um pacote requer reinicialização, a TUI para a fila e notifica
- **Grupos** para organizar e filtrar apps por categoria
- **Múltipla seleção** com `Space` para operar em vários apps de uma vez
- **Redimensionamento** do painel dividido arrastando com o mouse
- **Senha sudo** via modal seguro — sem expor no log ou no processo
- **Dry-run** (`--dry-run`) para ver o que seria executado sem rodar nada
- **Export** de snapshot do estado atual em JSON
- **Config padrão via XDG** — sem precisar passar caminho se `~/.config/settupper/packages.yaml` existir

---

## Instalação

### Via script

Linux/macOS:

```bash
curl -fsSL https://raw.githubusercontent.com/devbaraus/settupper/main/install.sh | sh
```

Windows PowerShell:

```powershell
irm https://raw.githubusercontent.com/devbaraus/settupper/main/install.ps1 | iex
```

Windows CMD:

```cmd
powershell -NoProfile -ExecutionPolicy Bypass -Command "irm https://raw.githubusercontent.com/devbaraus/settupper/main/install.ps1 | iex"
```

Por padrão, os scripts buscam a última release no GitHub, detectam sistema operacional e arquitetura, baixam o binário compilado correspondente e instalam em:

- Linux/macOS: `~/.local/bin/settupper`
- Windows: `%LOCALAPPDATA%\Programs\settupper\bin\settupper.exe`

Depois disso, se o diretório de instalação estiver no seu `PATH`, basta rodar:

```bash
settupper
```

Para instalar uma versão específica:

```bash
curl -fsSL https://raw.githubusercontent.com/devbaraus/settupper/main/install.sh | SETTUPPER_VERSION=v0.1.3 sh
```

No PowerShell:

```powershell
$env:SETTUPPER_VERSION = "v0.1.3"; irm https://raw.githubusercontent.com/devbaraus/settupper/main/install.ps1 | iex
```

Se o terminal não encontrar o comando `settupper`, adicione o diretório de instalação ao `PATH`. No Linux/macOS:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

### Manual

```bash
# Rodar diretamente sem instalar (recomendado para experimentar)
cargo run

# Instalar a release v0.1.3 como ferramenta
cargo install --git https://github.com/devbaraus/settupper --tag v0.1.3 --bin settupper --locked --force

# Desenvolvimento local
git clone https://github.com/devbaraus/settupper
cd settupper
cargo build --release
./target/release/settupper examples/packages.yaml
```

---

## Uso

```bash
# Abre a TUI com seu arquivo de config
settupper meus-pacotes.yaml

# Usa config padrão em ~/.config/settupper/packages.yaml
settupper

# Mostra o que seria executado sem rodar nada
settupper --dry-run meus-pacotes.yaml
```

---

## Teclas

| Tecla       | Ação                                                  |
|-------------|-------------------------------------------------------|
| `Space`     | Selecionar / desselecionar item                       |
| `Escape`    | Limpar seleção                                        |
| `i`         | Instalar selecionado(s)                               |
| `u`         | Atualizar selecionado(s)                              |
| `d`         | Desinstalar selecionado(s)                            |
| `a`         | Smart: install ou update conforme status              |
| `Shift+A`   | Smart All: todos os apps visíveis (respeita deps)     |
| `r`         | Recarregar status                                     |
| `e`         | Exportar snapshot JSON                                |
| `q`         | Sair                                                  |

---

## Formato do arquivo de configuração

```yaml
version: 1

groups:
  - id: dev-tools
    name: Ferramentas de Desenvolvimento
  - id: runtimes
    name: Runtimes

apps:
  - id: git
    name: Git
    group: dev-tools
    description: Controle de versão
    check:
      default:
        - command -v git
      windows:
        - where git
    actions:
      install:
        ubuntu:
          - sudo apt-get install -y git
        fedora:
          - sudo dnf install -y git
        macos:
          - brew install git
        windows:
          - winget install --id Git.Git -e
      update:
        ubuntu:
          - sudo apt-get install --only-upgrade -y git
        fedora:
          - sudo dnf upgrade -y git
        macos:
          - brew upgrade git
        windows:
          - winget upgrade --id Git.Git -e
      uninstall:
        ubuntu:
          - sudo apt-get remove -y git
        fedora:
          - sudo dnf remove -y git
        macos:
          - brew uninstall git
        windows:
          - winget uninstall --id Git.Git -e

  - id: nvm
    name: NVM
    group: runtimes
    description: Node Version Manager
    depends_on:
      - git                   # git será instalado antes de nvm
    reboot_on:
      install: false
    check:
      default:
        - test -d "$HOME/.nvm"
    actions:
      install:
        default:
          - curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.3/install.sh | bash
```

### Campos disponíveis por app

| Campo         | Obrigatório | Descrição |
|---------------|-------------|-----------|
| `id`          | **sim**     | Identificador único (gerado a partir de `name` se omitido) |
| `name`        | **sim**     | Nome exibido na TUI |
| `description` | não         | Descrição curta |
| `group`       | não         | ID do grupo para filtrar na TUI |
| `depends_on`  | não         | Lista de IDs de apps que devem estar instalados antes |
| `reboot_on`   | não         | Mapa de ação → bool indicando se precisa de reboot |
| `check`       | não         | Comandos para verificar se está instalado (por distro ou `default`) |
| `actions.install` | não    | Comandos de instalação por distro |
| `actions.update`  | não    | Comandos de atualização por distro |
| `actions.uninstall` | não  | Comandos de remoção por distro |

### Distros suportadas como chave

`ubuntu`, `fedora`, `macos`, `windows`, `default` (fallback para qualquer distro)

---

## Stack

| Lib | Uso |
|-----|-----|
| [Ratatui](https://github.com/ratatui-org/ratatui) | Framework TUI |
| [Serde](https://github.com/serde-rs/serde) | Parser de config YAML |
| [Tokio](https://docs.rs/tokio/latest/tokio/) | Gerenciamento de dependências e execução |

---

## Disclaimer

Este projeto foi construído inteiramente através de **Vibe Coding** — uma prática onde o desenvolvedor descreve o que quer em linguagem natural e a IA (neste caso, [Claude](https://claude.ai) da Anthropic) escreve todo o código.

---

## Licença

MIT
