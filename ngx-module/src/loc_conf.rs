
use syn::DeriveInput;
use syn::Data;
use syn::Fields;
use syn::Ident;
use quote::Tokens;

pub fn expand_loc_conf(input: &DeriveInput) -> Result<Tokens,String>  {

    let result = match input.data {
        Data::Struct(ref s) => handle_loc_struct(&input, &s.fields, None),
        Data::Union(_) => panic!("doesn't work with unions yet"),
        Data::Enum(ref e) => panic!("doesn't work with enums yet")
    };
    let expanded = quote! {
        fn test()  {
            #result
        }
    };
    Ok(expanded)
}

fn handle_loc_struct(ast: &DeriveInput,
                  fields: &Fields,
                  variant: Option<&Ident>) -> Tokens
{
    match *fields {
        Fields::Named(ref fields) => {
            
            //directive(&ast, Some(&fields.named), true, variant)
             let fnames = fields.named.iter().map(|f| f.ident);
             quote! {
                0 # (
                    let #fnames = 2;
                )*
             }
        },
        Fields::Unit => {
            quote!(0)
        },
        Fields::Unnamed(ref fields) => {
            quote!(0)
        },
    }
}

