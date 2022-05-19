use std::rc::Rc;
use std::sync::Arc;
use crate::core::ProtocolID;


pub trait SummaryTrait: Sync + Send {
    fn get_request_ip(&self) -> Arc<String>;

    fn get_protocol_id(&self) -> ProtocolID;
    fn set_protocol_id(&mut self, p: ProtocolID);

    fn get_sequence_id(&self) -> Arc<String>;
    fn set_sequence_id(&self, seq_id: String);
}


pub struct Summary {
    request_ip: Arc<String>,
    sequence_id: Arc<String>,
    protocol_id: ProtocolID,
}

impl Summary {
    pub fn new(request_ip: Arc<String>, sequence_id: Arc<String>, protocol_id: ProtocolID) -> Self {
        Summary { request_ip, sequence_id, protocol_id }
    }
}


pub struct SummaryBuilder {}

impl SummaryTrait for Summary {
    fn get_request_ip(&self) -> Arc<String> {
        Arc::clone(&self.request_ip)
    }


    fn get_protocol_id(&self) -> ProtocolID {
        self.protocol_id
    }

    fn set_protocol_id(&mut self, p: ProtocolID) {
        self.protocol_id = p
    }

    fn get_sequence_id(&self) -> Arc<String> {
        Arc::clone(&self.sequence_id)
    }

    fn set_sequence_id(&self, seq_id: String) {
        todo!()
    }
}

unsafe impl Send for Summary {}