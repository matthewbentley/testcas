extern crate cas;
extern crate hyper;

use cas::{CasClient, ServiceResponse};
use hyper::Server;
use hyper::server::{Request, Response};
use hyper::uri::RequestUri;
use hyper::Url;
use hyper::status::StatusCode;
use hyper::header::Location;

fn main() {
    let base_url = "https://login.case.edu/cas/";
    let login_path = "login";
    let logout_path = "logout";
    let verify_path = "serviceValidate";
    let service_url = "http://test.case.edu:3000/complete";
    let cas = CasClient::new(base_url, login_path, logout_path, verify_path,
                             service_url).unwrap();

    Server::http("127.0.0.1:3000").unwrap()
        .handle(move |req: Request, mut res: Response| {
            let s = match req.uri {
                RequestUri::AbsolutePath(s) => s,
                RequestUri::AbsoluteUri(u) => u.serialize(),
                RequestUri::Authority(s) => s,
                RequestUri::Star => panic!("Nope!")
            };
            if s.as_str().starts_with("/complete") {
                let u = format!("{}{}", "http://localhost", &s);
                let url = Url::parse(&u).unwrap();
                let q = url.query_pairs().unwrap();
                let mut ticket = "".to_string();
                for i in q {
                    let (v, t) = i;
                    if v == "ticket" {
                        ticket = t;
                    }
                }
                match cas.verify_ticket(&ticket).unwrap() {
                    ServiceResponse::Success(user) => {
                        {
                            let mut s = res.status_mut();
                            *s = StatusCode::Found;
                        }
                        {
                            let mut h = res.headers_mut();
                            let loc = format!("/{}", user);
                            h.set::<Location>(Location(loc));
                        }
                        res.send(b"").unwrap();
                        return ();
                    }
                    ServiceResponse::Failure(user) => {
                        {
                            let mut s = res.status_mut();
                            *s = StatusCode::Found;
                        }
                        {
                            let mut h = res.headers_mut();
                            let loc = format!("/{}", user);
                            h.set::<Location>(Location(loc));
                        }
                        res.send(b"").unwrap();
                        return ();

                    }
                }
            } else if s == "/" {
                cas.login_redirect(res);
                return();
            } else {
                res.send(s.as_bytes());
            }
    });
}
