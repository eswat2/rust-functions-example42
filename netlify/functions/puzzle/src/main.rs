extern crate base64;
extern crate serde_json;

use aws_lambda_events::event::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use aws_lambda_events::encodings::Body;
use http::header::HeaderMap;
use lambda_runtime::{handler_fn, Context, Error};
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::time::Instant;
use sudoku::Sudoku;

#[tokio::main]
async fn main() -> Result<(), Error> {
    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();

    let func = handler_fn(my_handler);
    lambda_runtime::run(func).await?;
    Ok(())
}

pub(crate) async fn my_handler(event: ApiGatewayProxyRequest, _ctx: Context) -> Result<ApiGatewayProxyResponse, Error> {
  let start = Instant::now();
  let generated = Sudoku::generate_unique();
  let sudoku_line = generated.to_str_line();
  let time = start.elapsed().as_nanos() as u64;
  let puzzle = format!("{}", sudoku_line);
  let blanks = puzzle.matches(".").count();

  let sudoku = Sudoku::from_str_line(&puzzle).unwrap();
  let fin = Instant::now();
  let mut solved = 0;
  let mut line = "".to_string();

  if let Some(solution) = sudoku.solve_unique() {
    solved = fin.elapsed().as_nanos() as u64;
    line = format!("{}", solution);
  }

  let data = serde_json::json!({
    "metrics" : {
        "counts": {
            "blanks": blanks,
            "clues": 81 - blanks
        },
        "nanos": {
            "generate": time,
            "solve": solved
        }
    },
    "puzzle": puzzle,
    "ref": base64::encode(line)
  });

  let payload = serde_json::to_string(&data).unwrap();

    let resp = ApiGatewayProxyResponse {
        status_code: 200,
        headers: HeaderMap::new({ "Content-Type", "application/json" }),
        multi_value_headers: HeaderMap::new(),
        body: Some(Body::Text(format!(payload))),
        is_base64_encoded: Some(false),
    };

  Ok(resp)
}