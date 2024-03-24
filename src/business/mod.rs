use cbsk_socket::cbsk_base;
use crate::data::analysis_data::AnalysisData;
use crate::data::verify_data::VerifyData;

pub mod cbsk_write_trait;

/// check verify data is too short
macro_rules! verify_too_short {
    ($bytes:expr,$header:expr,$error_frame:expr) => {
        if $bytes.len() <= $header.len() {
            return VerifyData::too_short($error_frame,$bytes);
        }
    };
}

/// verify if bytes is cbsk frame
pub fn verify(mut bytes: Vec<u8>, header: &[u8]) -> VerifyData {
    verify_too_short!(bytes,header,Vec::new());
    // find first frame
    let h = header[0];
    let index = cbsk_base::match_some_return!(bytes.iter().position(|b| { b.eq(&h) }),{
        // header does not exist in bytes
        VerifyData::fail(bytes)
    });

    // get next verify frame
    let mut verify_data = bytes.drain(index..).collect::<Vec<u8>>();
    verify_too_short!(verify_data,header,bytes);

    // verify frame
    for (index, b) in verify_data.iter().skip(1).take(header.len() - 1).enumerate() {
        if b.ne(&header[index + 1]) {
            // verify fail, return data
            let next_verify = verify_data.drain(index..).collect();
            bytes.append(&mut verify_data);
            return VerifyData::next_verify(bytes, next_verify);
        }
    }

    // verify success, remove header frame and return
    VerifyData::new(bytes, verify_data.drain(header.len()..).collect())
}

/// check analysis data lt min len
macro_rules! analysis_min_len {
    ($len:expr,$min_len:expr,$block:expr) => {
        if $len < $min_len {
            return $block;
        }
    };
}

/// check analysis data gt max len
macro_rules! analysis_max_len {
    ($len:expr,$max_len:expr,$block:expr) => {
        if $len > $max_len {
            return $block;
        }
    };
}

/// data analysis
pub fn analysis(mut bytes: Vec<u8>, header: &[u8]) -> AnalysisData {
    // limit the minimum length to 3
    analysis_min_len!(bytes.len(),3,AnalysisData::next_verify(bytes));

    // get bytes of description length
    let len = usize::from(bytes[0]);
    // limit description length to 8, and check too short
    analysis_max_len!(len,8,AnalysisData::too_long(bytes[0],bytes.drain(1..).collect()));
    analysis_min_len!(bytes.len(),len + 1,AnalysisData::too_short(build_analysis_too_short(bytes,header.to_vec())));

    // obtain the actual data length based on the description length
    let data_len = &bytes[1..len + 1].iter().enumerate().map(|(i, v)| {
        256_usize.pow(i.try_into().unwrap_or_default()) * usize::from(*v)
    }).sum::<usize>();
    let all_len = data_len + len + 1;
    analysis_min_len!(bytes.len(),all_len,AnalysisData::too_short(build_analysis_too_short(bytes,header.to_vec())));

    // normal data length, obtain real data and next verify data
    let data = bytes.drain(len + 1..all_len).collect::<Vec<u8>>();
    bytes.drain(..len + 1);// not, bytes is verify data

    AnalysisData::success(data, bytes)
}

/// build analysis too short data, will add header to data, used for next data reception and verification
fn build_analysis_too_short(mut bytes: Vec<u8>, mut header: Vec<u8>) -> Vec<u8> {
    header.append(&mut bytes);
    header
}

/// encapsulation of data before sending
pub fn frame(mut bytes: Vec<u8>, header: &[u8]) -> Vec<u8> {
    let mut list = header.to_vec();
    list.append(&mut calc_data_len(&bytes));
    list.append(&mut bytes);

    list
}

/// calc data len
fn calc_data_len(bytes: &[u8]) -> Vec<u8> {
    let mut len = bytes.len();
    let mut list = Vec::new();

    while len > 255 {
        list.push(u8::try_from(len % 256).unwrap_or_default());
        len /= 256;
    }

    // if last len gt zero, add to len list
    if len > 0 {
        list.push(u8::try_from(len).unwrap_or_default());
    }
    // add first bytes is length of data length
    list.insert(0, u8::try_from(list.len()).unwrap_or_default());
    list
}
