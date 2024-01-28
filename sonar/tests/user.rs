#[tokio::test]
async fn list_users_empty() {
    let ctx = sonar::test::create_context_memory().await;
    let users = sonar::user_list(&ctx, sonar::ListParams::default())
        .await
        .unwrap();
    assert_eq!(users.len(), 0);
}

#[tokio::test]
async fn create_user_one() {
    let ctx = sonar::test::create_context_memory().await;
    let create = sonar::UserCreate {
        username: "User".parse().unwrap(),
        password: "password".to_string(),
        avatar: None,
    };
    let user = sonar::user_create(&ctx, create).await.unwrap();
    assert_eq!(user.username.as_str(), "User");
}

#[tokio::test]
async fn list_users_one() {
    let ctx = sonar::test::create_context_memory().await;
    let user = sonar::test::create_user(&ctx, "User").await;
    let users = sonar::user_list(&ctx, sonar::ListParams::default())
        .await
        .unwrap();
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].id, user.id);
}

#[tokio::test]
async fn list_users_two() {
    let ctx = sonar::test::create_context_memory().await;
    let user1 = sonar::test::create_user(&ctx, "User1").await;
    let user2 = sonar::test::create_user(&ctx, "User2").await;
    let users = sonar::user_list(&ctx, sonar::ListParams::default())
        .await
        .unwrap();
    assert_eq!(users.len(), 2);
    assert_eq!(users[0].id, user1.id);
    assert_eq!(users[1].id, user2.id);
}

#[tokio::test]
#[should_panic]
async fn create_two_same_username() {
    let ctx = sonar::test::create_context_memory().await;
    sonar::test::create_user(&ctx, "User").await;
    sonar::test::create_user(&ctx, "User").await;
}

#[tokio::test]
async fn get_user_one() {
    let ctx = sonar::test::create_context_memory().await;
    let user = sonar::test::create_user(&ctx, "User").await;
    let user2 = sonar::user_get(&ctx, user.id).await.unwrap();
    assert_eq!(user.id, user2.id);
    assert_eq!(user.username, user2.username);
}
