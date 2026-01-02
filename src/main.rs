use nalgebra::DMatrix;
use prost::Message;
use std::convert::TryFrom;

pub mod myprotobuf {
    include!(concat!(env!("OUT_DIR"), "/myprotobuf.rs"));
}

use myprotobuf::Detection as DetectionProto;
use myprotobuf::DetectionList as DetectionListProto;
use myprotobuf::MatrixRow as MatrixRowProto;

#[derive(Debug, PartialEq)]
struct Detection {
    timestamp: f64,
    number: usize,
    x: f64,
    y: f64,
    mat: DMatrix<f64>,
}

#[derive(Debug, PartialEq)]
struct DetectionList {
    uid: String,
    dets: Vec<Detection>,
}

impl From<&Detection> for DetectionProto {
    fn from(d: &Detection) -> Self {
        DetectionProto {
            timestamp: d.timestamp,
            number: d.number as u64,
            x: d.x,
            y: d.y,
            mat_data: d
                .mat
                .row_iter()
                .map(|row| MatrixRowProto {
                    values: row.iter().cloned().collect(),
                })
                .collect(),
        }
    }
}

impl TryFrom<DetectionProto> for Detection {
    type Error = String;

    fn try_from(proto: DetectionProto) -> Result<Self, Self::Error> {
        let rows = proto.mat_data.len();
        let cols = if rows > 0 {
            proto.mat_data[0].values.len()
        } else {
            0
        };

        let data: Vec<f64> = proto.mat_data.into_iter().flat_map(|r| r.values).collect();

        if data.len() != rows * cols {
            return Err("Matrix dimensions do not match data length".to_string());
        }

        let mat = DMatrix::from_row_iterator(rows, cols, data.into_iter());

        Ok(Detection {
            timestamp: proto.timestamp,
            number: proto.number as usize,
            x: proto.x,
            y: proto.y,
            mat,
        })
    }
}

impl From<&DetectionList> for DetectionListProto {
    fn from(list: &DetectionList) -> Self {
        DetectionListProto {
            uid: list.uid.clone(),
            dets: list.dets.iter().map(DetectionProto::from).collect(),
        }
    }
}

impl From<&DetectionListProto> for DetectionList {
    fn from(proto: &DetectionListProto) -> Self {
        DetectionList {
            uid: proto.uid.clone(),
            dets: proto
                .dets
                .iter()
                .map(|d| Detection::try_from(d.clone()).unwrap())
                .collect(),
        }
    }
}

fn main() {
    let original_list = DetectionList {
        uid: "sensor-001".to_string(),
        dets: vec![
            Detection {
                timestamp: 161000.0,
                number: 1,
                x: 10.5,
                y: 20.5,
                mat: DMatrix::from_vec(2, 2, vec![1.0, 2.0, 3.0, 4.0]),
            },
            Detection {
                timestamp: 161001.0,
                number: 2,
                x: 15.0,
                y: 25.0,
                mat: DMatrix::identity(3, 3),
            },
        ],
    };

    println!("Original: {:?}", original_list);

    let proto_list = DetectionListProto::from(&original_list);

    let mut buf = Vec::new();
    buf.reserve(proto_list.encoded_len());
    proto_list.encode(&mut buf).unwrap();

    println!("Encoded {} bytes: {:?}", buf.len(), buf);

    let decoded_proto = DetectionListProto::decode(&buf[..]).unwrap();

    let decoded_list = DetectionList::from(&decoded_proto);

    println!("Decoded: {:?}", decoded_list);

    assert_eq!(original_list, decoded_list);
    println!("Success! Original and Decoded lists match.");
}
