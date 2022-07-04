use shaku::{module, Component, Interface, HasComponent};
use crate::extension::HttpExtension;

module! {
    HttpModule{
        components=[HttpExtension],
        providers=[]
    }
}


#[cfg(test)]
mod tests {


    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
    #[test]
    fn test_http_module(){

    }
}