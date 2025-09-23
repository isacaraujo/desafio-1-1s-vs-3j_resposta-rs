# [Resposta] Desafio Técnico: Performance e Análise de Dados via API

https://github.com/codecon-dev/desafio-1-1s-vs-3j

> Este README utiliza português vulgar de São Paulo. Por favor, não se prenda aos vícios de linguagem. O importante é o racional por traz do projeto.

## Introdução

Resposta ao [Desafio CodeCon 1 Dev Sr. VS 3 Dev Jr.](https://www.youtube.com/watch?v=AFtRYXJVO-4).

Utilizarei este projeto para utilizar um caso de uso real e entregar uma API em Rust.

### Stack

- [Rust ~= 1.85](https://www.rust-lang.org/tools/install)
- [Rocket ~= 0.5](https://rocket.rs/guide/v0.5/)

### Plugins

Para melhorar o desenvolvimento estou utilizando os seguintes plugins:

- [asdf](https://asdf-vm.com/): Amo este version manager
- [CodeLLDB | vscode](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb): 
- [editorconfig](https://editorconfig.org/): não perca tempo configurando editor com tab ou spaces. Este plugin resolve tudo pra você!
- [rust | vscode](https://marketplace.visualstudio.com/items?itemName=1YiB.rust-bundle): Sou novo no Rust, então preciso de um help no desenvolvimento.
- [awesome rust](https://github.com/rust-unofficial/awesome-rust): O primeiro awesome rust que encontrei. So far so good.

## Começando o projeto

Clone o projeto:

```sh
$ git clone git@github.com:isacaraujo/desafio-1-1s-vs-3j_resposta-rs.git
$ cd desafio-1-1s-vs-3j_resposta-rs
```

Faça um build do projeto - não há um `npm install` ou equivalente :/

```
$ cargo build
```

### Linter e Beautification

Todas as ferramentas abaixo são built-in do Rust <3.

#### Rust Check

Use o _rust check_ para verificar warnings e afins:

```sh
$ cargo check
 ```

#### Clippy

Use o `clippy` para fazer a análise estática do código (excelente ferramenta):

```sh
$ cargo clippy
```

E para corrigir:

```sh
$ cargo clippy --fix --bin "challengeresult"
```

#### Formatter

Para formatar de deixar o código em conformidade:

```sh
$ cargo fmt
```

### Build

O rust é uma linguagem compilada, logo, ele entrga um binário no final do projeto. Para o ambiente de desenvolvimento, podemos gerar um código não otimizado, que prioriza o a velocidade de compilação:

```
$ cargo build
```

Já para rodar em produção, utilizamos a o pofile `release`:

```
$ cargo build --release
```

Para saber mais: https://doc.rust-lang.org/book/ch14-01-release-profiles.html

## Por que Rust?

Trabalho com desenvolvimento desde 2010 - façam as contas. Desde esta época já entreguei projetos em PHP (mais do que gostaria - a linguagem me persegue), java, node (js e ts), golang, ruby, python, alguns frontends também, Java, Kotlin (Android) e Objective-C/C/C++ (iOS) e também IoT (c).

As melhores experiências que tive foi com o desenvolvimento em Objective-C. Como trata-se de uma linguagem compilada, foi a que mais me trouxe uma visão de gerenciamento de memória e entrega de performance, uma vez que na época, um iPhone tinha muito pouca performance comparado aos PCs do mesmo tempo.

Já no backend, trabalhei com Go que também me ofereceu uma performance muito destoante de quaisquer outras linguagens com o mesmo propósito (com excessão de Java, mas aí é uma conversa para um outro forum). O problema do Go é sempre ter a impressão de estar trabalhando em uma espécie de sandbox, super dependente de um [garbage collector](https://tip.golang.org/doc/gc-guide), mas até então sempre dependi de um runtime (php, python, ...), então estas features eram o menor dos problemas.

Até que ...

Agora temos os LLM's que podem dar um boost no aprendizado e nos auxiliar a revisar o código produzido, why not?

Pensei em C, C++ (que tem um bom approach para desenvolvimento web), mas ando lendo tanto sobre Rust que resolvi largar a preguiça e começar a aprender.

Bicho, é difícil, viu!?

Tem muita novidade.

É uma linguagem SEM GARGABE COLLECTOR (isso só é uma vantagem se você tiver uma aplicação que precise de muito, mas muito fine tunning) e que você não precisa se preocupar com o uso de memória (mais ou menos hehehe). 

Agora, como ele faz isso? Em compile time! O compilador é muito chato. É isso! Se tu conseguir compilar o código já é meio caminho andado.

O objetivo é mandar o LLM produzir código pra mim, mas pra isso funcionar, antes eu preciso dominar a tecnologia. Por isso estou investindo tanto neste projeto.

Este desafio está sendo um grande laboratório pra mim. A idéia, diferente do desafio proposto, não é entregar nada em 3 horas (que honestamente é um tempo bem aceitável pra fazer em qualquer linguagem com runtime, diga-se de passagem - desafio muito simples), mas sim entregar e entender o que eu estou entregando.

Vou fazer vários resultados, mudar o código, voltar, escrever testes, enfim, fazer tudo o que eu preciso pra dominar a linguagem.

Pensou se eu fico bom nisso? Imagina entregar uma API (simples, claro), desenvolvida com a mesma facilidade e velocidade que tenho em outras linguagens, só que utilizando 100MiB de RAM e .25 de CPU max pra 1MM requests/dia (11/12 req/s throughput) e latência abaixo dos 200ms!?

A idéia é essa: se me perguntarem quais linguagens eu conheço, eu só respondo "RUST"! hehehehe
