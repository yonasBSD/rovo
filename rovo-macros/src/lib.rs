use proc_macro::TokenStream;
use quote::quote;

mod parser;
use parser::parse_rovo_function;

/// Macro that generates OpenAPI documentation from doc comments.
///
/// # Example
///
/// ```ignore
/// /// Get a single Todo item.
/// ///
/// /// Retrieve a Todo item by its ID.
/// ///
/// /// @response 200 Json<TodoItem> A single Todo item.
/// /// @response 404 () Todo was not found.
/// #[rovo]
/// async fn get_todo(State(app): State<AppState>, Path(todo): Path<SelectTodo>) -> impl IntoApiResponse {
///     // ...
/// }
/// ```
#[proc_macro_attribute]
pub fn rovo(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = item.clone();

    match parse_rovo_function(input.into()) {
        Ok((func_item, doc_info)) => {
            let func_name = &func_item.name;
            let docs_func_name = quote::format_ident!("{}_docs", func_name);

            let title = doc_info.title.as_deref().unwrap_or("");
            let description = doc_info.description.as_deref().unwrap_or("");

            // Generate response setters if we have doc comments
            let response_code_setters = if doc_info.responses.is_empty() {
                // No responses specified - generate a minimal docs function
                vec![]
            } else {
                doc_info.responses.iter().map(|resp| {
                    let code = resp.status_code;
                    let response_type = &resp.response_type;
                    let desc = &resp.description;

                    // Check if there's an explicit example for this status code
                    if let Some(example) = doc_info.examples.iter().find(|e| e.status_code == code) {
                        let example_code = &example.example_code;
                        quote! {
                            .response_with::<#code, #response_type, _>(|res| {
                                res.description(#desc)
                                    .example(#example_code)
                            })
                        }
                    } else {
                        // No explicit example, just add the description
                        quote! {
                            .response_with::<#code, #response_type, _>(|res| {
                                res.description(#desc)
                            })
                        }
                    }
                }).collect()
            };

            let output = quote! {
                #func_item

                pub fn #docs_func_name(op: aide::transform::TransformOperation) -> aide::transform::TransformOperation {
                    op.summary(#title)
                        .description(#description)
                        #(#response_code_setters)*
                }
            };

            output.into()
        }
        Err(err) => {
            let err_msg = err.to_string();
            quote! {
                compile_error!(#err_msg);
            }
            .into()
        }
    }
}
