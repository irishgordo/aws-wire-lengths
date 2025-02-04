use crate::prelude::*;

pub async fn do_key(mut l: Level<Stuff>) -> Result<()> {
    l.cmda("list", "ls", "list SSH keys", cmd!(do_key_ls))?;
    l.cmd("create", "create new SSH key", cmd!(do_key_create))?;
    l.cmda("destroy", "rm", "destroy an SSH key", cmd!(do_key_rm))?;

    sel!(l).run().await
}

async fn do_key_rm(mut l: Level<Stuff>) -> Result<()> {
    l.usage_args(Some("KEYNAME..."));

    let a = args!(l);
    let s = l.context();

    if a.args().is_empty() {
        bail!("specify which key name(s) to delete");
    }

    for name in a.args().iter() {
        s.ec2()
            .delete_key_pair(ec2::DeleteKeyPairRequest {
                key_name: Some(name.to_string()),
                ..Default::default()
            })
            .await?;
        println!("deleted {}", name);
    }

    Ok(())
}

async fn do_key_create(mut l: Level<Stuff>) -> Result<()> {
    l.usage_args(Some("KEYNAME"));
    l.reqopt("o", "", "output file name for created key", "PEMFILE");

    let a = args!(l);
    let s = l.context();

    if a.args().len() != 1 {
        bail!("just one key name");
    }

    /*
     * Open the target file without overwriting an existing file.
     */
    let path = a.opts().opt_str("o").unwrap();
    let mut f = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(&path)?;

    let key_name = a.args().get(0).unwrap().to_string();

    let res = s
        .ec2()
        .create_key_pair(ec2::CreateKeyPairRequest {
            key_name,
            ..Default::default()
        })
        .await?;

    if let Some(mat) = res.key_material.as_deref() {
        f.write_all(mat.as_bytes())?;
        f.flush()?;
    } else {
        bail!("no key material in response: {:#?}", res);
    }

    if let Some(fp) = res.key_fingerprint.as_deref() {
        eprintln!("fingerprint = {}", fp);
    }

    Ok(())
}

async fn do_key_ls(mut l: Level<Stuff>) -> Result<()> {
    l.add_column("name", 20, true);
    l.add_column("fingerprint", 59, true);
    l.add_column("id", 21, false);

    let a = no_args!(l);
    let mut t = a.table();
    let s = l.context();

    let res = s
        .ec2()
        .describe_key_pairs(ec2::DescribeKeyPairsRequest {
            ..Default::default()
        })
        .await?;

    let x = Vec::new();
    for kp in res.key_pairs.as_ref().unwrap_or(&x) {
        let mut r = Row::default();

        r.add_stror("id", &kp.key_pair_id, "?");
        r.add_stror("name", &kp.key_name, "-");
        r.add_stror("fingerprint", &kp.key_fingerprint, "-");

        t.add_row(r);
    }

    print!("{}", t.output()?);

    Ok(())
}
