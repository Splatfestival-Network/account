use crate::Pool;

pub async fn next_pid(pool: &Pool) -> i32{
    loop {
        let next_pid = sqlx::query!("SELECT nextval('pid_counter') as pid")
            .fetch_one(pool)
            .await
            .expect("unable to get next pid")
            .pid
            .expect("unable to get next pid") as i32;

        let already_exists = sqlx::query!(
            "SELECT EXISTS(select 1 from users where pid = $1)",
            next_pid
        ).fetch_one(pool)
                .await
                .ok()
                .map(|v| v.exists)
                .flatten()
                .unwrap_or(true);

        if !already_exists{
            break next_pid;
        }
    }



}