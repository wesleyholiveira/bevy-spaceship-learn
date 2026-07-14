# Bevy Spaceship

Projeto mínimo em Rust e Bevy 0.19 para iniciar um jogo de nave. A aplicação abre uma janela, renderiza um bloco como nave e permite movimentá-lo com `W`, `A`, `S` e `D`.

O projeto usa Cargo Workspace com `resolver = "3"`, Rust 2024 e crates separadas por responsabilidade, sem adicionar camadas ou abstrações desnecessárias.

## Requisitos

- Rust 1.95 ou superior
- Cargo
- Dependências nativas do Bevy para o sistema operacional

Consulte a configuração oficial do Bevy para instalar as dependências nativas da sua plataforma:

https://bevy.org/learn/quick-start/getting-started/setup/

## Estrutura

```text
bevy-spaceship/
├── apps/
│   └── spaceship/
├── crates/
│   ├── core/
│   ├── input/
│   ├── render/
│   └── ui/
├── .dockerignore
├── .gitignore
├── Cargo.toml
├── Dockerfile
├── README.md
└── rustfmt.toml
```

## Responsabilidade das crates

### `apps/spaceship`

Ponto de entrada e composition root da aplicação. Configura a janela e registra os plugins das demais crates.

### `crates/core`

Contém o estado e as regras básicas de gameplay:

- componente da nave;
- posição lógica;
- velocidade de movimento;
- intenção de movimento;
- ordem de execução dos sistemas;
- atualização da posição independente de FPS.

Não contém detalhes visuais ou leitura direta de dispositivos de entrada.

### `crates/input`

Converte o estado do teclado em uma intenção de movimento. Atualmente reconhece `W`, `A`, `S` e `D`.

### `crates/render`

Contém a apresentação 2D:

- criação da câmera;
- criação do sprite da nave;
- cor de fundo;
- sincronização da posição lógica com o `Transform` visual.

### `crates/ui`

Contém os elementos de interface. Atualmente exibe a indicação dos controles no canto da janela.

## Ordem dos sistemas

Os sistemas são organizados nesta sequência:

```text
Input → Simulation → Presentation
```

A entrada é lida antes da simulação, e a apresentação é sincronizada somente depois da atualização do estado.

## Executar em desenvolvimento

```bash
cargo run --package spaceship
```

## Build de release

```bash
cargo build --release --package spaceship
```

O executável será criado em:

```text
target/release/spaceship
```

No Windows, o arquivo terá o nome `spaceship.exe`.

## Formatação e análise estática

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check --workspace --all-targets
```

Para aplicar a formatação:

```bash
cargo fmt --all
```

## Testes

Executar todos os testes do workspace:

```bash
cargo test --workspace
```

Executar os testes de uma crate específica:

```bash
cargo test --package spaceship-core
cargo test --package spaceship-input
```

### Testes unitários

Testes unitários devem ficar próximos do código testado em um módulo `tests`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describes_expected_behavior() {
        assert_eq!(2 + 2, 4);
    }
}
```

Prefira extrair regras determinísticas para funções pequenas e testá-las sem inicializar a aplicação inteira. As crates `core` e `input` já possuem exemplos desse formato.

### Testes de integração

Quando um teste precisar validar a API pública de uma crate, crie um arquivo dentro de `tests` na própria crate:

```text
crates/core/tests/movement.rs
```

O teste deve importar a crate como um consumidor externo:

```rust
use spaceship_core::SHIP_SPEED;

#[test]
fn exposes_the_default_ship_speed() {
    assert!(SHIP_SPEED > 0.0);
}
```

Para sistemas Bevy, crie uma `App`, registre somente os recursos e plugins necessários, execute `app.update()` e consulte o `World` para verificar o resultado.

## Docker

O Dockerfile usa build multi-stage:

- a etapa `builder` compila o binário em release;
- a etapa `runtime` contém somente o executável e as bibliotecas Linux necessárias para janela e renderização.

### Build da imagem

```bash
docker build --tag bevy-spaceship .
```

### Executar com X11 no Linux

Aplicações gráficas precisam acessar o servidor de janelas e a GPU do host.

Para Intel ou AMD com `/dev/dri`:

```bash
xhost +local:docker

docker run --rm \
  --env DISPLAY \
  --volume /tmp/.X11-unix:/tmp/.X11-unix:rw \
  --device /dev/dri \
  bevy-spaceship

xhost -local:docker
```

Para NVIDIA com NVIDIA Container Toolkit:

```bash
xhost +local:docker

docker run --rm \
  --env DISPLAY \
  --volume /tmp/.X11-unix:/tmp/.X11-unix:rw \
  --gpus all \
  bevy-spaceship

xhost -local:docker
```

A imagem é Linux. A execução gráfica pelo Docker Desktop no Windows ou macOS depende de uma solução externa de exibição, como WSLg ou um servidor X. Para desenvolvimento desktop local, `cargo run` costuma ser a opção mais simples.

## Cargo.lock

Como este workspace produz uma aplicação executável, o `Cargo.lock` gerado pelo primeiro comando Cargo deve ser versionado. Ele não está listado no `.gitignore`.

## Adicionando uma crate

Crie a biblioteca dentro de `crates`:

```bash
cargo new --lib crates/combat
```

Adicione o caminho em `workspace.members`, registre a dependência em `workspace.dependencies` e consuma-a somente nas crates que realmente precisarem dela.

Evite dependências circulares. Funcionalidades de apresentação podem depender do estado do `core`, mas o `core` não deve depender de `render` ou `ui`.
