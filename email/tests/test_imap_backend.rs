#[cfg(feature = "imap-backend")]
#[test]
fn test_imap_backend() {
    use concat_with::concat_line;
    use mail_builder::MessageBuilder;
    use pimalaya_email::{
        AccountConfig, Backend, Email, Flag, ImapAuthConfig, ImapBackend, ImapConfig, PasswdConfig,
        Tpl, DEFAULT_INBOX_FOLDER,
    };
    use pimalaya_secret::Secret;

    env_logger::builder().is_test(true).init();

    let config = AccountConfig {
        email_reading_decrypt_cmd: Some(
            "gpg --decrypt --quiet --recipient-file ./tests/keys/bob.key".into(),
        ),
        email_reading_verify_cmd: Some("gpg --verify --quiet".into()),
        ..AccountConfig::default()
    };

    let imap = ImapBackend::new(
        config.clone(),
        ImapConfig {
            host: "127.0.0.1".into(),
            port: 3143,
            ssl: Some(false),
            starttls: Some(false),
            insecure: Some(true),
            login: "bob@localhost".into(),
            auth: ImapAuthConfig::Passwd(PasswdConfig {
                passwd: Secret::new_raw("password"),
            }),
            ..ImapConfig::default()
        },
    )
    .unwrap();

    // setting up folders

    for folder in imap.list_folders().unwrap().iter() {
        imap.purge_folder(&folder.name).unwrap();

        match folder.name.as_str() {
            DEFAULT_INBOX_FOLDER => (),
            folder => imap.delete_folder(folder).unwrap(),
        }
    }

    imap.add_folder("Sent").unwrap();
    imap.add_folder("Trash").unwrap();
    imap.add_folder("Отправленные").unwrap();

    // checking that an email can be built and added
    let email = MessageBuilder::new()
        .from("alice@localhost")
        .to("bob@localhost")
        .subject("subject")
        .text_body(concat_line!(
            "<#part type=text/plain>",
            "Hello, world!",
            "<#/part>",
        ));
    let email = Tpl::from(email.write_to_string().unwrap())
        .compile()
        .unwrap()
        .write_to_vec()
        .unwrap();

    let id = imap
        .add_email("Sent", &email, &("seen".into()))
        .unwrap()
        .to_string();

    // checking that the added email exists
    let emails = imap.get_emails("Sent", vec![&id]).unwrap();
    let interpreter = Email::get_tpl_interpreter(&config);
    let tpl = emails
        .to_vec()
        .first()
        .unwrap()
        .to_read_tpl(
            interpreter
                .hide_all_headers()
                .show_headers(["From", "To"])
                .hide_part_markup()
                .hide_multipart_markup(),
        )
        .unwrap();
    let expected_tpl = concat_line!(
        "From: alice@localhost",
        "To: bob@localhost",
        "",
        "Hello, world!",
        "",
    );

    assert_eq!(*tpl, expected_tpl);

    // checking that the envelope of the added email exists
    let sent = imap.list_envelopes("Sent", 0, 0).unwrap();
    assert_eq!(1, sent.len());
    assert_eq!("alice@localhost", sent[0].from.addr);
    assert_eq!("subject", sent[0].subject);

    // checking that the email can be copied
    imap.copy_emails("Sent", "Отправленные", vec![&sent[0].id])
        .unwrap();
    let sent = imap.list_envelopes("Sent", 0, 0).unwrap();
    let sent_ru = imap.list_envelopes("Отправленные", 0, 0).unwrap();
    let trash = imap.list_envelopes("Trash", 0, 0).unwrap();
    assert_eq!(1, sent.len());
    assert_eq!(1, sent_ru.len());
    assert_eq!(0, trash.len());

    // checking that the email can be marked as deleted then expunged
    imap.mark_emails_as_deleted("Отправленные", vec![&sent_ru[0].id])
        .unwrap();
    let sent = imap.list_envelopes("Sent", 0, 0).unwrap();
    let sent_ru = imap.list_envelopes("Отправленные", 0, 0).unwrap();
    let trash = imap.list_envelopes("Trash", 0, 0).unwrap();
    assert_eq!(1, sent.len());
    assert_eq!(1, sent_ru.len());
    assert_eq!(0, trash.len());
    assert!(sent_ru[0].flags.contains(&Flag::Deleted));

    imap.expunge_folder("Отправленные").unwrap();
    let sent_ru = imap.list_envelopes("Отправленные", 0, 0).unwrap();
    assert_eq!(0, sent_ru.len());

    // checking that the email can be moved
    imap.move_emails("Sent", "Отправленные", vec![&sent[0].id])
        .unwrap();
    let sent = imap.list_envelopes("Sent", 0, 0).unwrap();
    let sent_ru = imap.list_envelopes("Отправленные", 0, 0).unwrap();
    let trash = imap.list_envelopes("Trash", 0, 0).unwrap();
    assert_eq!(0, sent.len());
    assert_eq!(1, sent_ru.len());
    assert_eq!(0, trash.len());

    // checking that the email can be deleted
    imap.delete_emails("Отправленные", vec![&sent_ru[0].id])
        .unwrap();
    let sent = imap.list_envelopes("Sent", 0, 0).unwrap();
    let sent_ru = imap.list_envelopes("Отправленные", 0, 0).unwrap();
    let trash = imap.list_envelopes("Trash", 0, 0).unwrap();
    assert_eq!(0, sent.len());
    assert_eq!(0, sent_ru.len());
    assert_eq!(1, trash.len());

    imap.delete_emails("Trash", vec![&trash[0].id]).unwrap();
    let trash = imap.list_envelopes("Trash", 0, 0).unwrap();
    assert_eq!(1, trash.len());
    assert!(trash[0].flags.contains(&Flag::Deleted));

    imap.expunge_folder("Trash").unwrap();
    let trash = imap.list_envelopes("Trash", 0, 0).unwrap();
    assert_eq!(0, trash.len());

    // clean up

    imap.purge_folder("INBOX").unwrap();
    imap.delete_folder("Sent").unwrap();
    imap.delete_folder("Trash").unwrap();
    imap.delete_folder("Отправленные").unwrap();
    imap.close().unwrap();
}
