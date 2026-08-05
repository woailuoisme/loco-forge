use crate::app::App;
use loco_rs::app::Hooks;
use loco_rs::prelude::*;

pub struct RouteList;

#[async_trait]
impl Task for RouteList {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "route:list".to_string(),
            detail: "Display all registered HTTP API routes in a Laravel route:list style table".to_string(),
        }
    }

    async fn run(&self, app_context: &AppContext, _vars: &task::Vars) -> Result<()> {
        let routes = App::routes(app_context);

        println!("\n+------------+---------------------------------------------------------+");
        println!("| {:<10} | {:<55} |", "Method", "URI");
        println!("+------------+---------------------------------------------------------+");

        for route in routes.collect() {
            let methods = route.actions.iter().map(|a| a.as_str()).collect::<Vec<_>>().join("|");
            println!("| {:<10} | {:<55} |", methods, route.uri);
        }
        println!("+------------+---------------------------------------------------------+\n");
        Ok(())
    }
}
