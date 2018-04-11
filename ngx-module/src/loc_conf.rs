
use syn::DeriveInput;
use syn::Data;
use syn::Fields;
use syn::Field;
use syn::Ident;
use syn::punctuated;
use quote::Tokens;

pub fn expand_loc_conf(input: &DeriveInput) -> Result<Tokens,String>  {

    let result = match input.data {
        Data::Struct(ref s) => handle_loc_struct(&input, &s.fields, None),
        Data::Union(_) => panic!("doesn't work with unions yet"),
        Data::Enum(ref e) => panic!("doesn't work with enums yet")
    };
    Ok(result)
}

fn handle_loc_struct(ast: &DeriveInput,
                  fields: &Fields,
                  variant: Option<&Ident>) -> Tokens
{
    match *fields {
        Fields::Named(ref fields) => {
            
            directive(&ast, Some(&fields.named), true, variant)
        },
        Fields::Unit => {
            directive(&ast, None, false, variant)
        },
        Fields::Unnamed(ref fields) => {
            directive(&ast, Some(&fields.unnamed), false, variant)
        },
    }
}

fn directive(ast: &DeriveInput,
            fields: Option<&punctuated::Punctuated<Field, Token![,]>>,
            named: bool, variant: Option<&Ident>) -> Tokens
{
    let name = &ast.ident;
    quote! {
        printf!("field name: {}",name)
    }
}

/*
#[cfg(test)]
mod tests {

    
    #[repr(C)]
    #[NgxLocConf]
    pub struct test_loca_conf_t {
        #[loc_conf(name="topic")]
        pub topic: ngx_str_t,
        pub destination_service: ngx_str_t
    }

    
}
*/

