use actix_web::{cookie::Cookie, http::header::ContentType, HttpRequest, HttpResponse};

pub fn login_form(request: HttpRequest) -> HttpResponse {
    let error_html = match request.cookie("_flash") {
        None => "".into(),
        Some(cookie) => {
            format!("<p><i>{}</i></p>", cookie.value())
        }
    };
    let response = HttpResponse::Ok()
        .content_type(ContentType::html())
        .del_cookie(&Cookie::new("_flash", ""))
        .body(format!(
            r#"
<!DOCTYPE html>
<html lang="en">
<head>
	<meta http-equiv="content-type" content="text/html; charset=utf-8">
	<title>Login</title>
</head>
<body>
    {}
	<form action="/login" method="post">
		<label>Username
			<input type="text" placeholder="Enter Username" name="username">
		</label>
		<label>Password
			<input type="password" placeholder="Enter Password" name="password">
		</label>
		<button type="submit">Login</button>
	</form>
</body>
</html>
           "#,
            error_html
        ));
    response
}
