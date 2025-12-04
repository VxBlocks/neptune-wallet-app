use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use clap::Parser;
use neptune_privacy::config_models::data_directory::DataDirectory;
use neptune_privacy::config_models::network::Network;

use crate::rpc::client::RestRpcClient;
use crate::rpc::Output;
use crate::rpc::SendToAddressParams;
use crate::wallet::fake_archival_state::generate_snapshot;
#[derive(Parser)]
enum WalletCli {
    RUN(RunArgs),
    SEND(SendArgs),
    HISTORY(HistoryArgs),
    SNAPSHOT(SnapshotArgs),
}

#[derive(clap::Args)]
struct GlobalArgs {
    #[clap(long)]
    data_dir: Option<PathBuf>,
    #[clap(long)]
    rpc: Option<String>,
}

#[derive(clap::Args)]
struct RunArgs {
    #[clap(flatten)]
    global: GlobalArgs,
}

#[derive(clap::Args)]
struct SendArgs {
    #[clap(flatten)]
    global: GlobalArgs,
    #[clap(long)]
    amount: String,
    #[clap(long)]
    address: String,
    #[clap(long, default_value = "0.01")]
    fee: String,
    #[clap(long)]
    priority_fee: Option<String>,
    #[clap(long, default_value = FEE_ADDRESS)]
    fee_address: String,
}

#[derive(clap::Args)]
struct SnapshotArgs {
    #[clap(long, short)]
    output_dir: PathBuf,
    #[clap(long, default_value = "0")]
    start: u64,
    #[clap(long)]
    end: u64,
    #[clap(flatten)]
    global: GlobalArgs,
}

const FEE_ADDRESS:&'static str = "nolgam1nurfm22evhpscn5ddwgwa96z0048454c84hwapmvqq6rqqwqx4w34kudq6q5adjvgch8f8v9dsfz3h0vk60npzya04248umqq2xs9n9cznxzl92nh65k6pg60jesff6wu77l8e3c2h8yyjtwwd9kz00m6z7nl5vxk5929q34837shxn4x5t6p9wgheljlfs3kp7lnrl2z0an80y50lwzm704svvpw3ze5k9fkccttuhunjn96cr3jcgt80qggj5x9ltta5z3qmyxhxxmz9ns7kddcrtun0mfd5fz2d05xnkhjzp3pphc83jytrecc437gf7e9czqh9qfhw5000f43ghyc2dfa5vcl38rwzax27kuv0e0gtkj7q2ar3dt0q6y32fdp9nhtm9l4crg7ud7w6vlg28ncns5q4f86teneuu8ezs2zur30gscw5qk9dgmter2nzryph5k2r68k5xf5pf7lkjas9km6eu6jjl2ujfjv5572xqrdrymm3mne6gptpvg54qxfwp3kkm45fvc5knjecsv7w5dfx82u9kcl5mrdd39k8dgc6gddty49f4yy32nfczhxq0k5dx5qmyet273mz6ggthrtvsxtteg3ceg366pnhmgaplejmjgq7qyyc0vz43ecvry8k7p7ddysqutxgpm6w950mzcxcppe5rm6pkjv9tv5uxyx3kz8lpd744udfc8h0575lfkxuwfp4y3uf9nu3fzj8x2r4gt8y3wtwdlf3flldp0m289jc3lh0dv9372dxk7fddx3ns9acfz7cdxsluucxnrn7e8p7lx5h3ngztft68ae5fcnplekay90kvnqjnxr3e80q4xl0nufucchr66p6swa2gkptf85304wwjktllz7f2sswpx3qkpld8mku900jz0g6e2q9y806enem49qud89uqu6z8d98v9sux5anr2v88hr80jqz7t7g4dcj5spgnc0l996lrq0hfswzfwldx7klsxk82zlpfzwpfgkmu3gkdyqnh9salfwrckn95tk0k0kyhrkchhaplehldfj5wf6dnkhapaxhzwfzu8gglp2rf3jtpx7ew3hlq6yqtxtrfxu0ctwsycj9eqccnlpg77mjs292t39kz4n99vjd2yejuxztk4828yk2wk5urejc3fd00gwqmcxl4k2pw85vmxrvv8n9dv6amcgkmuhgfzfcy3wm0p5yhtvdhs4l0447au6x7kwdhmuxjgk7x80gtdmgd74zswdw0jkngwef2zctxnuktxp4e5fqftgw0yplq0d3lcrcqg6q3rw5ljc654adhee53xmmeaazg0avtzkt2q0ngsq8xuxxcax8u2x9zhcxjltcsewhe7ffzqrkznv3z3vuhar4whazsergmymz4jx2d3l8qwrlhcducztkkeygm8luwnrmh2fcrpkg79gj34u88e72ljt94aapkn5uunu457h2kc3czpgekjl2wjyuz9wcpyfk3z22xx7lx7etchn5mfqxpvjf63wcy0sd9qap8mwnmfzs5j4zh9jv8n8jdwvjyk5d3x0j42cdvh5zhq00g429j0vrvm8097vfq2fg2axhrzfuy6qv97swl39dm3q859guyk4pqv9a82kz5wgnvs84l9g3g5wjf9z888spenf97ddaprkxvxluhg268hst8jgfa78t4nrqklgvw6f630nt4yrsddwahmfcfux9gmt0zjyg9vkfrfct8qtg9lehrvgmwq4e7h6ys6r34l2xn82fy2ey5wwq0jn6vk52vugmzlpgc0aywltxqzn7dvz6dlec98en9f482vdmhf33th0k5nrpwq3qj6xg7ve09nna3kp3ff4nhknt4etqhzauc8v2047yl72yefh4zddc6g9s4ye4hvukulhhu37gqrll7qyg0sx6gtgalwgwcc50gd00m90vzca8mxykdqjhfesxre99ahmfcpa2xtqftzlvu8ag55wqm84rqapa06774v876lms39y5mx0r67mus4n45crh4j99f6wptmcmy9q8hqlnl8qgvxetx3ce3kla74uwuleh7jkzdpafgcvl7amv0s8usgg6z2nr3utc4xg5qgzaf5zw3tjnak72e0ptl86k5d2667pkzauq35c7x83tms2ysev6x20h5am89qu6mm77f8f7cemtd4hhxh4qp6ae55krpst59656mqzpzc8uup42mxrarc298n7y86ekgrgft3nkasfa30u9w50dxt6gx3rpyvpgsyv8nz3d0dhzgdtkt7gxd6nj02awyesdmncj0pwzdp59gh2c09rqfm7x8t7le70ej2dd7ncq2z2qwl0cphu8ds5hxzegur3mlrrqx0zdvmje79s86ads9v6srn2skztz7mlr47f2xs43tt2eejx0j66ukqusg2ltjjxe79efggq022u9j8dqd6qcuedrfhhm8rqg6na9rcuq35aqn40q4llseyrdz68x5enuyt7yhk3d3kqxwjfullcrqhtc82vzraw0pdgjxpjtxgjvrqeqfdn7j9ck57w2u5dppfuvkk52cc3mn28nnshn87j84vfd3tdkqu9wl037yn49l829gftaky623476hw4wc7x26al8q7mfsg56pmzlyzdmgqsa33r37k0thurnjasahp3c9z5mwk3zgtgtfvj2qydgz5su6wvewhh7yeqft8z2ze4j99qha32wagywmjuqhtff3v7wpdmrcu84zmlxd5zhf5lngp4t070uup93w7lv95uk6ckhrqq4fx8epcuynh6qwh86a03nvnjf7vxvmkae2l2qzu24pjz8wdtwqs87pfdhzcwj29ruzh9ag54zqe8qzw46azds62ug7qxgf3z00rgu5q28newruew6pcvv7w7uvs9fzchha5awsfk2xfjtyu3ml5y98m2fs7peusgwv9r78uy8w6stzgc9prtsa57l03l7sfhakkt40va06uwva5qc6vy8mztwkdw2z69xpzuf4qaz9rk83wtjqjj5xvxp4xjpeple9dxgxp0tqhqzt2f8t8r03dn0vx9tl6tnh7mn6k2tnatwqkjx0csz5fj7a3g4fs07rv2p2hxag0hc8p29hx4skh0xp6x2y6afwrs5jx8hagl8pm320wwwfeh2zsernkgul5jhpy2ea5tjf934z6qgwsxezex94w935z2txr8gw3fcsrpp4m94nmwmap3pe6xyw5qlz7yyjg9merzckv6lxe5k8rtysn7fgzy3f5ug99hzq29gpllklmja7sdjg2wwgxee6m5nqercjx48cta7qp4q6hyerdts4fc5ly0hemn9rnygwng4hckqc7le3u7jpemgjxjc4rudzdekqllkg88k9p3m0gadjm4s2ha5r42p0cv5ss44n7kfyzw4scpyjw0alt2rmuwckvezejusxsxdqu6c8ad0ja7fqh2e4";

#[derive(clap::Args)]
struct HistoryArgs {
    #[clap(flatten)]
    global: GlobalArgs,
}

pub async fn run() {
    let cli = WalletCli::parse();

    match cli {
        WalletCli::RUN(args) => {
            run_server(args).await.unwrap();
        }
        WalletCli::SEND(args) => {
            send(args).await.unwrap();
        }
        WalletCli::HISTORY(args) => {
            history(args).await.unwrap();
        }
        WalletCli::SNAPSHOT(args) => {
            snapshot(args).await.unwrap();
        }
    }
}

async fn run_server(args: RunArgs) -> Result<()> {
    let data_dir = DataDirectory::get(args.global.data_dir, Network::Main)?.root_dir_path();
    info!("data_dir: {}", data_dir.to_string_lossy());
    let config = crate::config::Config::new(&data_dir).await.unwrap();
    let config = Arc::new(config);

    crate::service::manage(config.clone());

    if let Some(rpc) = args.global.rpc {
        crate::rpc_client::node_rpc_client().set_rest_server(rpc);
    } else {
        crate::rpc_client::node_rpc_client()
            .set_rest_server(config.get_remote_rest().await.unwrap());
    }

    if !crate::rpc::commands::has_password().await.unwrap() {
        crate::rpc::commands::set_password("".to_string(), "".to_string())
            .await
            .unwrap();
    }

    crate::rpc::commands::try_password().await.unwrap();

    let wallets = crate::rpc::commands::get_wallets().await.unwrap();
    if wallets.is_empty() {
        // ask user to input mnemonic
        let mnemonic: String = dialoguer::Input::new()
            .with_prompt("Enter your mnemonic(divide by space):")
            .interact_text()?;

        let num_keys: u64 = dialoguer::Input::new()
            .with_prompt("Enter the number of keys:")
            .default(25)
            .interact_text()?;

        let start_height = dialoguer::Input::new()
            .with_prompt("Enter the start height:")
            .default(0)
            .interact_text()?;

        crate::rpc::commands::add_wallet(
            "default".to_string(),
            mnemonic,
            num_keys,
            start_height,
            false,
        )
        .await
        .expect("Failed to add wallet");
    }

    crate::rpc::commands::run_rpc_server().await.unwrap();

    let token = crate::rpc::commands::get_token().await.unwrap();

    println!("Wallet server started. Token: {}", token);
    write_token(&data_dir, &token).await?;

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            crate::rpc::commands::stop_rpc_server().await.unwrap();
        }
    }

    Ok(())
}

async fn send(args: SendArgs) -> Result<()> {
    let data_dir = DataDirectory::get(args.global.data_dir, Network::Main)?.root_dir_path();
    let token = read_token(&data_dir).await?;
    let rest_client = RestRpcClient::new(token);

    let mut outputs = vec![Output {
        address: args.address,
        amount: args.amount,
    }];

    if let Some(fee) = args.priority_fee {
        outputs.push(Output {
            address: args.fee_address,
            amount: fee,
        });
    }

    let params = SendToAddressParams {
        outputs,
        fee: args.fee,
        input_rule: None,
        inputs: vec![],
    };
    let txid = rest_client.send(&params).await?;
    println!("txid: {}", txid);

    Ok(())
}

async fn history(args: HistoryArgs) -> Result<()> {
    let data_dir = DataDirectory::get(args.global.data_dir, Network::Main)?.root_dir_path();
    let token = read_token(&data_dir).await?;

    let rest_client = RestRpcClient::new(token);

    let history = rest_client.history().await?;

    for h in history {
        println!("{:?}", h);
    }

    Ok(())
}

async fn snapshot(args: SnapshotArgs) -> Result<()> {
    crate::rpc_client::node_rpc_client().set_rest_server(
        args.global
            .rpc
            .unwrap_or("https://nptwallet.vxb.ai".to_string()),
    );

    generate_snapshot(
        &PathBuf::from(args.output_dir),
        Network::Main,
        (args.start..args.end).into(),
    )
    .await?;
    Ok(())
}

async fn write_token(data_dir: &PathBuf, token: &str) -> std::io::Result<()> {
    tokio::fs::write(data_dir.join("token"), token).await
}

async fn read_token(data_dir: &PathBuf) -> std::io::Result<String> {
    tokio::fs::read_to_string(data_dir.join("token")).await
}
