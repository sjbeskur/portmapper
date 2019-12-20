use mapper;


#[cfg(test)]
mod tests{

    #[test]
    fn mac_to_string_test(){
        
        let test = "184.39.235.100.148.192";
        println!("--->{}", mapper::convert_to_hex(test));
        let mac = eui48::MacAddress::new([184,39,235,100,148,192]);
            
        assert_eq!(true, true);    
    }

    #[test]
    fn to_do(){
        let add_some_test = true;
        assert_eq!(add_some_test, true);
    }


}