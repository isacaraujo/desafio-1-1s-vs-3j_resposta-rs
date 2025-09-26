#[macro_use]
extern crate rocket;

use chrono::Local;
use rocket::State;
use rocket::form::Form;
use rocket::serde::json::Json;
use rocket::tokio::time::Instant;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicPtr, Ordering};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
struct TeamProject {
    name: String,
    completed: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
struct UserTeam {
    name: String,
    leader: bool,
    projects: Vec<TeamProject>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
struct UserLog {
    date: String,
    action: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
struct User {
    id: String,
    name: String,
    age: u8,
    score: u16,
    active: bool,
    country: String,
    team: UserTeam,
    logs: Vec<UserLog>,
}

#[derive(Serialize, PartialEq, Debug)]
#[serde(crate = "rocket::serde")]
struct CreateUsersResp {
    message: String,
    user_count: usize,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "rocket::serde")]
struct GetSuperusersResp {
    timestamp: String,
    execution_time_ms: u128,
    user_count: usize,
    data: Vec<User>,
}

struct Root {
    users: AtomicPtr<Vec<User>>,
}

impl Root {
    fn new() -> Root {
        /* Esta parte eu precisei perguntar pro Claude (Antropic).
         * O rolê aqui é que se eu fizer `let users = Vec::new()` o
         * compilador irá liberar a memória quando a função acabar.
         * Então o `users` vai apontar para uma memória 'já liberada' (confia
         * em mim - eu perdi horas pra entender isso).
         * Para resolver o Claude me explicou que eu precisava guardar
         * o Vec na 'HEAP'. O problema é que o Rust não gerencia a HEAP,
         * então o 'memory release' fica com você bro (veja o código de update)!
         * Eu fiz desta forma pra aprender mas depois eu vou pensar em mudar.
         *
         * Saiba mais:
         * https://web.mit.edu/rust-lang_v1.25/arch/amd64_ubuntu1404/share/doc/rust/html/book/first-edition/the-stack-and-the-heap.html
         */
        let users = Box::new(Vec::new());

        Root {
            users: AtomicPtr::new(Box::into_raw(users)),
        }
    }

    #[cfg(test)]
    fn from_users(users: Vec<User>) -> Root {
        let users_box = Box::new(users);

        Root {
            users: AtomicPtr::new(Box::into_raw(users_box)),
        }
    }

    fn update(&self, new_users: Vec<User>) {
        let new_box = Box::new(new_users);
        let new_ptr = Box::into_raw(new_box);

        let old_ptr = self.users.swap(new_ptr, Ordering::AcqRel);

        if !old_ptr.is_null() {
            /* A partir de agora é só gambiarra:
             * Quando você usa o Box::new, o compiler
             * cria um novo endereço de memória e te
             * dá! Agora, quando você cria um Box a
             * partir de um endereço de memória já
             * existente, o compiler pega
             * este endereço de memória e TRANSFERE O
             * OWNERSHIP PARA ESTE NOVO CONTEXTO - leiam
             * sobre Rust Ownership. É uma viagem a parte,
             * mas necessário!
             * Aí como o endereço de memória não será mais
             * utilizado, ele será varrido!
             * É assim que se libera memória da Heap!
             * Poderia ter um `release()`, né!? Anyway...
             * */
            unsafe {
                let _ = Box::from_raw(old_ptr);
            }
        }
    }

    fn get_users(&self) -> Vec<User> {
        let ptr = self.users.load(Ordering::Acquire);

        if ptr.is_null() {
            return Vec::new();
        }

        unsafe { (*ptr).clone() }
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let ptr = self.users.load(Ordering::Acquire);
        if !ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(ptr);
            }
        }
    }
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[derive(FromForm, Debug)]
struct Upload {
    file: String,
}

#[post("/users", data = "<upload>")]
async fn post_users(
    upload: Form<Upload>,
    root: &State<Root>,
) -> std::io::Result<Json<CreateUsersResp>> {
    /* FOI MUITO DIFÍCIL FAZER ESTE MÉTODO!
     * Tem algumas formas de processar um multipart request:
     * - Podemos processar o request Raw - aí precisaríamos
     *   de uma biblioteca pra poder converter os form fields
     *   em algum dado válido
     * - Utilizar o esquema de form. O problema é que aparentemente
     *   só funciona via `TempFile` (vou adicionar um TODO pra
     *   testar usando String ou byte_array hehehe).
     *
     * Pois bem, um outro desafio foi configurar o arquivo Rocket.toml
     * para o framework aceitar arquivos muito grandes (o sample com
     * 100k usuários tem ~65MiB).
     *
     * Bom, o resultado do código são as gambiarras abaixo (acredite,
     * sem ajuda de LLM hahaha - talvez por isso não ficou tão bom).
     */
    let users: Vec<User> = serde_json::from_str(&upload.file)?;

    let users_len = users.len();

    /* Ah, aqui foi uma prova dos 30 hehehe (pedi ajuda
     * ao Claude).
     * Fiz o codigo abaixo somente com o `let users` e
     * o root.update(users), mas o código quebrava no
     * `users_count` da resposta.
     * A desgraça aconteceu pois o compiler faz o 'move'
     * da variável `users` no root.update() - não passamos
     * o &users, então não é borrow, né!?
     * Pois bem, ao fazer users.len() de uma variável que
     * não existe, o código quebra!
     * Tinha algumas formas de resolver, inclusive refatorando
     * o código pra funcionar tudo via borrow (muita mudança),
     * ou então salvar o users_len em uma variável antes de
     * chamar o root.update() - achei mais inteligente.
     */
    root.update(users);

    Ok(Json(CreateUsersResp {
        message: String::from("Arquivo recebido com sucesso"),
        user_count: users_len,
    }))
}

#[get("/superusers")]
fn get_superusers(root: &State<Root>) -> Json<GetSuperusersResp> {
    // Filtro: score >= 900 e active = true
    // Retorna os dados e o tempo de processamento da requisição.
    let start_time = Instant::now();

    let users = root.get_users();

    /* Este código abaixo tem um glitch:
     * Cara, perdi muito tempo tentando resolver,
     * mas deu certo!
     * Inicialmente eu tentei muitas vezes fazer
     * `users.iter().filter(..).collect()`, e tentava
     * converter o código todo pra usar `&` (isso é anotação
     * de borrow, não de ref - no final é a msm coisa, mas pra rust
     * é importante entender).
     * O problema é que eu teria que reescrever tudo, inclusive
     * os DTO do serializers, e o 'serde' eu descobri que não
     * lida bem com refs.
     * Aí eu pedi ajuda pro meu amigo Claude e tudo se resolveu
     * hahahaha - brincadeira. Eu sempre pergunto o que eu to
     * fazendo de errado antes mesmo da solução, e se ele sugerir
     * algo, me explicar como.
     * O `iter` itera sobre as refs dos itens. Então se você chamar
     * `collect()``, ele vai te devolver um Vec das refs, e não
     * da estrutura.
     * O que precisamos fazer é uma cópia do resultado do filter para
     * então chamar o `collect()`.
     * Ah, o serde ainda não serializa Iterators (infelizmente).
     * */
    let superusers: Vec<User> = users
        .iter()
        .filter(|u| u.score >= 900 && u.active)
        .cloned()
        .collect();

    println!("users len: {}; capacity: {}", users.len(), users.capacity());

    /* NOTA DO EDITOR:
     * `start_time.elapsed()` <3 - achei fofo hahaha
     */
    let elapsed_time = start_time.elapsed();

    Json(GetSuperusersResp {
        timestamp: format!("{:?}", Local::now()),
        execution_time_ms: elapsed_time.as_millis(),
        user_count: superusers.len(),
        data: superusers,
    })
}

#[derive(Serialize, Deserialize, Debug)]
struct CountrySummary {
    country: String,
    total: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct TopCountriesResp {
    timestamp: String,
    execution_time_ms: u128,
    countries: Vec<CountrySummary>,
}

#[get("/top-countries")]
async fn get_topcountries(root: &State<Root>) -> std::io::Result<Json<TopCountriesResp>> {
    // Agrupa os superusuários por país.
    // Retorna os 5 países com maior número de superusuários.
    let start_time = Instant::now();

    let users = root.get_users();

    /* Aqui foi uma tentativa de fazer um map/reduce
     * no estilo Rust.
     * Achei bem bom. Neste ponto eu já estava um pouco mais
     * familiarizado com o esquema de ownership, então resolver
     * aqui foi mais google e conhecimendo adquirido.
     */
    let summary: HashMap<String, usize> = users.iter().fold(HashMap::new(), |mut acc, u| {
        let def = 0;
        let val = acc.get(&u.country).unwrap_or(&def);
        acc.insert(u.country.clone(), val + 1);
        acc
    });

    let mut sorted: Vec<(String, usize)> = summary.into_iter().collect();
    sorted.sort_by(|(_, av), (_, bv)| bv.cmp(av));

    let countries: Vec<CountrySummary> = sorted[0..5]
        .iter()
        .map(|(c, t)| CountrySummary {
            country: c.clone(),
            total: *t,
        })
        .collect();

    Ok(Json(TopCountriesResp {
        timestamp: format!("{:?}", Local::now()),
        execution_time_ms: start_time.elapsed().as_millis(),
        countries,
    }))
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct TeamInsight {
    team: String,
    total_members: usize,
    leaders: usize,
    completed_projects: usize,
    active_percentage: f32,

    /* Maaaaaaanooo, demorei muito pra sacar
     * este esquema. O skip salvou aqui */
    #[serde(skip_serializing, skip_deserializing)]
    active_count: usize,
}

impl TeamInsight {
    fn new() -> Self {
        TeamInsight {
            team: String::from(""),
            total_members: 0,
            leaders: 0,
            completed_projects: 0,
            active_percentage: 0.0,
            active_count: 0,
        }
    }

    /* Tem um glitch aqui hehehe
     * O &mut self é esquema pra alterar o estado
     * da variável. Se não colocar, quebra!
     * Ah, o borrow checker vale tbm pra método,
     * tá!?
     */
    fn update_with_user(&mut self, u: &User) {
        self.total_members += 1;

        if u.active {
            self.active_count += 1;
        }

        /* Este código aqui abaixo tá feio bagarai....
         * Mais um TODO pra dar uma refatoradazin dele,
         * né!? Um `math_round()` ou algo assim */
        let scale_factor = 10.0;
        let active_pct = self.active_count as f32 / self.total_members as f32 * 100.0;

        self.active_percentage = (active_pct * scale_factor).trunc() / scale_factor;

        if u.team.leader {
            self.leaders += 1;
        }

        for p in u.team.projects.iter() {
            if p.completed {
                self.completed_projects += 1;
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct TeamInsightsResp {
    timestamp: String,
    execution_time_ms: u128,
    teams: Vec<TeamInsight>,
}

#[get("/team-insights")]
async fn get_team_insights(root: &State<Root>) -> std::io::Result<Json<TeamInsightsResp>> {
    // Agrupa por team.name.
    // Retorna: total de membros, líderes, projetos
    // concluídos e % de membros ativos.
    let start_time = Instant::now();

    let users = root.get_users();

    let summary: HashMap<String, TeamInsight> = users.iter().fold(HashMap::new(), |mut acc, u| {
        let def = TeamInsight::new();
        let mut insight = acc.get(&u.team.name).unwrap_or(&def).to_owned();

        insight.update_with_user(u);

        /* Olha, eu não sei se fazer isso é a melhor opção -
         * inserir algo já inserido.
         * Vou perder a vergonha e perguntar pro clause
         * se há uma forma melhor de melhorar este insert
         */
        acc.insert(u.team.name.clone(), insight.clone());

        acc
    });

    let teams: Vec<TeamInsight> = summary.values().cloned().collect();

    Ok(Json(TeamInsightsResp {
        timestamp: format!("{:?}", Local::now()),
        execution_time_ms: start_time.elapsed().as_millis(),
        teams,
    }))
}

#[derive(Serialize, Deserialize, Debug)]
struct ActiveUserLogin {
    date: String,
    total: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct ActiveUsersResp {
    timestamp: String,
    execution_time_ms: u128,
    logins: Vec<ActiveUserLogin>,
}

#[get("/active-users-per-day?<min>")]
async fn get_active_users_per_day(
    min: Option<u16>,
    root: &State<Root>,
) -> std::io::Result<Json<ActiveUsersResp>> {
    // Conta quantos logins aconteceram por data.
    // Query param opcional: ?min=3000 para filtrar dias com pelo menos 3.000 logins.
    let start_time = Instant::now();

    let users = root.get_users();

    let summary: HashMap<String, usize> = users.iter().fold(HashMap::new(), |mut acc, u| {
        for l in u.logs.iter() {
            let def = 0;
            let login_count = acc.get(&l.date).unwrap_or(&def);

            /* Mais uma gambiarra do insert. Vou ver
             * se há como melhorar!
             */
            acc.insert(l.date.clone(), login_count + 1);
        }
        acc
    });

    let min_ = min.unwrap_or(0) as usize;

    /* Será que faz sentido usar ActiveUserLogin::new()???
     * Acho que é preciosismo (vou deixar no TODO com nota
     * de frescura check)
     */
    let logins: Vec<ActiveUserLogin> = summary
        .into_iter()
        .filter(|(_, v)| v >= &min_)
        .map(|(date, total)| ActiveUserLogin { date, total })
        .collect();

    Ok(Json(ActiveUsersResp {
        timestamp: format!("{:?}", Local::now()),
        execution_time_ms: start_time.elapsed().as_millis(),
        logins,
    }))
}

#[derive(Serialize, Debug)]
struct RouteMetric {
    status: u16,
    time_ms: u128,
    valid_response: bool,
}

#[derive(Serialize, Debug)]
struct EvaluationResp {
    tested_endpoints: HashMap<String, RouteMetric>,
}

struct Evaluation;

const BASE_URL: &str = "http://localhost:8000";

/* Esta classe Evaluation definitivamente precisa ser refatorada!
 * Na pressa eu criei uma função evaluate pra cada endpoint. Na velocidade
 * achei prático fazer o copy/paste. Mas tá olhar essa grosseria aqui
 */
impl Evaluation {
    async fn evaluate_superusers(
        mut endpoints: HashMap<String, RouteMetric>,
    ) -> std::io::Result<HashMap<String, RouteMetric>> {
        let resp = reqwest::get(format!("{}/superusers", BASE_URL))
            .await
            .unwrap();

        let status = resp.status();

        let body = resp.json::<GetSuperusersResp>().await.unwrap();

        endpoints.insert(
            String::from("/superusers"),
            RouteMetric {
                status: status.as_u16(),
                time_ms: body.execution_time_ms,
                valid_response: true,
            },
        );

        Ok(endpoints)
    }

    async fn evaluate_topcountries(
        mut endpoints: HashMap<String, RouteMetric>,
    ) -> std::io::Result<HashMap<String, RouteMetric>> {
        let resp = reqwest::get(format!("{}/top-countries", BASE_URL))
            .await
            .unwrap();

        let status = resp.status();

        let body = resp.json::<TopCountriesResp>().await.unwrap();

        endpoints.insert(
            String::from("/top-countries"),
            RouteMetric {
                status: status.as_u16(),
                time_ms: body.execution_time_ms,
                valid_response: true,
            },
        );

        Ok(endpoints)
    }

    async fn evaluate_team_insights(
        mut endpoints: HashMap<String, RouteMetric>,
    ) -> std::io::Result<HashMap<String, RouteMetric>> {
        let resp = reqwest::get(format!("{}/team-insights", BASE_URL))
            .await
            .unwrap();

        let status = resp.status();

        let body = resp.json::<TeamInsightsResp>().await.unwrap();

        endpoints.insert(
            String::from("/team-insights"),
            RouteMetric {
                status: status.as_u16(),
                time_ms: body.execution_time_ms,
                valid_response: true,
            },
        );

        Ok(endpoints)
    }

    async fn evaluate_active_users_per_day(
        mut endpoints: HashMap<String, RouteMetric>,
    ) -> std::io::Result<HashMap<String, RouteMetric>> {
        let resp = reqwest::get(format!("{}/active-users-per-day", BASE_URL))
            .await
            .unwrap();

        let status = resp.status();

        let body = resp.json::<ActiveUsersResp>().await.unwrap();

        endpoints.insert(
            String::from("/active-users-per-day"),
            RouteMetric {
                status: status.as_u16(),
                time_ms: body.execution_time_ms,
                valid_response: true,
            },
        );

        Ok(endpoints)
    }
}

#[get("/evaluation")]
async fn get_evaluation() -> std::io::Result<Json<EvaluationResp>> {
    // Ele deve executar uma autoavaliação dos principais
    // endpoints da API e retornar um relatório de pontuação.
    //
    // A avaliação deve testar:
    //
    // Se o status retornado é 200
    // O tempo em milisegundos de resposta
    // Se o retorno é um JSON válido
    // Esse endpoint pode rodar scripts de teste embutidos
    // no próprio projeto e retornar um JSON com os resultados.
    // Ele será utilizado para validar a entrega de forma
    // automática e rápida.
    let mut tested_endpoints = HashMap::new();

    /* Início do show de horrores */
    tested_endpoints = Evaluation::evaluate_superusers(tested_endpoints)
        .await
        .unwrap();
    tested_endpoints = Evaluation::evaluate_topcountries(tested_endpoints)
        .await
        .unwrap();
    tested_endpoints = Evaluation::evaluate_team_insights(tested_endpoints)
        .await
        .unwrap();
    tested_endpoints = Evaluation::evaluate_active_users_per_day(tested_endpoints)
        .await
        .unwrap();
    /* Fim do show de horrores */

    Ok(Json(EvaluationResp { tested_endpoints }))
}

#[launch]
fn rocket() -> _ {
    rocket::build().manage(Root::new()).mount(
        "/",
        routes![
            index,
            post_users,
            get_superusers,
            get_topcountries,
            get_team_insights,
            get_active_users_per_day,
            get_evaluation,
        ],
    )
}

#[cfg(test)]
mod tests {
    use std::{any::type_name, fs::File, io::Read, path::Path};

    use rocket::{Build, Rocket};

    use super::*;

    fn type_of<T>(_: T) -> &'static str {
        type_name::<T>()
    }

    fn _load_sample(sample_name: &str) -> String {
        let formatted_path = format!("./samples/{}.json", sample_name);
        let sample_path = Path::new(&formatted_path);

        let mut buf = String::new();

        File::open(sample_path)
            .unwrap()
            .read_to_string(&mut buf)
            .unwrap();

        buf
    }

    fn _load_fixture_users(fixture_name: &str) -> serde_json::Result<Vec<User>> {
        let buf = _load_sample(fixture_name);

        serde_json::from_str(&buf)
    }

    fn _build_app_with_empty_root() -> Rocket<Build> {
        rocket::build().manage(Root::new())
    }

    fn _build_app_with_fixture(fixture_name: &str) -> Rocket<Build> {
        let users = _load_fixture_users(fixture_name).unwrap();
        rocket::build().manage(Root::from_users(users))
    }

    fn _use_root_state(rocket: &Rocket<Build>) -> &State<Root> {
        State::get(rocket).unwrap()
    }

    #[tokio::test]
    async fn test_post_users() {
        let rocket = _build_app_with_empty_root();
        let root = _use_root_state(&rocket);
        let buf = _load_sample("usuarios_10");

        let upload = Form::from(Upload { file: buf });

        let resp = post_users(upload, root).await.unwrap();

        assert_eq!(
            resp.0,
            CreateUsersResp {
                message: "Arquivo recebido com sucesso".to_owned(),
                user_count: 10,
            }
        );

        assert_eq!(root.get_users().len(), 10);
    }

    #[test]
    fn test_get_superusers() {
        let rocket = _build_app_with_fixture("usuarios_10");
        let state = _use_root_state(&rocket);

        let resp = get_superusers(state).0;

        let expect_user = r#"
            {
                "id": "c460b871-77ec-46f1-9127-22ea6989b0bc",
                "name": "Clarice Porto",
                "age": 52,
                "score": 1040,
                "active": true,
                "country": "Argentina",
                "team": {
                    "name": "Frontend Avengers",
                    "leader": true,
                    "projects": [
                        {
                            "name": "Sistema Interno",
                            "completed": true
                        }
                    ]
                },
                "logs": [
                    {
                        "date": "2025-03-28",
                        "action": "login"
                    },
                    {
                        "date": "2025-03-29",
                        "action": "login"
                    },
                    {
                        "date": "2025-03-30",
                        "action": "login"
                    },
                    {
                        "date": "2025-03-27",
                        "action": "login"
                    },
                    {
                        "date": "2025-03-30",
                        "action": "login"
                    }
                ]
            }
        "#;

        assert_eq!(type_of(resp.timestamp), "alloc::string::String");
        assert_eq!(type_of(resp.execution_time_ms), "u128");
        assert_eq!(resp.user_count, 1);
        assert_eq!(resp.data[0], serde_json::from_str(expect_user).unwrap());
    }
}
