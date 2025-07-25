use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Result as IoResult, Error, ErrorKind};
use std::net::Ipv4Addr;
use std::collections::HashMap;

// Types de requêtes DNS selon RFC 1035
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DnsRecordType {
    A = 1,      // Adresse IPv4
    NS = 2,     // Name Server
    CNAME = 5,  // Canonical Name
    PTR = 12,   // Pointer
    MX = 15,    // Mail Exchange
    AAAA = 28,  // Adresse IPv6
}

impl DnsRecordType {
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            1 => Some(DnsRecordType::A),
            2 => Some(DnsRecordType::NS),
            5 => Some(DnsRecordType::CNAME),
            12 => Some(DnsRecordType::PTR),
            15 => Some(DnsRecordType::MX),
            28 => Some(DnsRecordType::AAAA),
            _ => None,
        }
    }
}

// Classes DNS
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DnsClass {
    IN = 1,  // Internet
}

impl DnsClass {
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            1 => Some(DnsClass::IN),
            _ => None,
        }
    }
}

// En-tête DNS (12 octets selon RFC 1035)
#[derive(Debug, Clone)]
pub struct DnsHeader {
    pub id: u16,            // Identifiant de transaction
    pub qr: bool,           // Query (0) ou Response (1)
    pub opcode: u8,         // Code d'opération (0 = requête standard)
    pub aa: bool,           // Authoritative Answer
    pub tc: bool,           // Truncated
    pub rd: bool,           // Recursion Desired
    pub ra: bool,           // Recursion Available
    pub z: u8,              // Réservé (doit être 0)
    pub rcode: u8,          // Response Code
    pub qdcount: u16,       // Nombre de questions
    pub ancount: u16,       // Nombre de réponses
    pub nscount: u16,       // Nombre d'enregistrements d'autorité
    pub arcount: u16,       // Nombre d'enregistrements additionnels
}

impl DnsHeader {
    pub fn new() -> Self {
        DnsHeader {
            id: rand::random(),
            qr: false,
            opcode: 0,
            aa: false,
            tc: false,
            rd: true,
            ra: false,
            z: 0,
            rcode: 0,
            qdcount: 0,
            ancount: 0,
            nscount: 0,
            arcount: 0,
        }
    }

    // Sérialiser l'en-tête DNS en bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // ID (2 bytes)
        bytes.write_u16::<BigEndian>(self.id).unwrap();

        // Flags (2 bytes)
        let mut flags = 0u16;
        if self.qr { flags |= 1 << 15; }
        flags |= (self.opcode as u16 & 0xF) << 11;
        if self.aa { flags |= 1 << 10; }
        if self.tc { flags |= 1 << 9; }
        if self.rd { flags |= 1 << 8; }
        if self.ra { flags |= 1 << 7; }
        flags |= (self.z as u16 & 0x7) << 4;
        flags |= self.rcode as u16 & 0xF;
        bytes.write_u16::<BigEndian>(flags).unwrap();

        // Counts (8 bytes)
        bytes.write_u16::<BigEndian>(self.qdcount).unwrap();
        bytes.write_u16::<BigEndian>(self.ancount).unwrap();
        bytes.write_u16::<BigEndian>(self.nscount).unwrap();
        bytes.write_u16::<BigEndian>(self.arcount).unwrap();

        bytes
    }

    // Désérialiser l'en-tête DNS depuis bytes
    pub fn from_bytes(data: &[u8]) -> IoResult<Self> {
        if data.len() < 12 {
            return Err(Error::new(ErrorKind::InvalidData, "Header trop court"));
        }

        let mut cursor = Cursor::new(data);

        let id = cursor.read_u16::<BigEndian>()?;
        let flags = cursor.read_u16::<BigEndian>()?;
        let qdcount = cursor.read_u16::<BigEndian>()?;
        let ancount = cursor.read_u16::<BigEndian>()?;
        let nscount = cursor.read_u16::<BigEndian>()?;
        let arcount = cursor.read_u16::<BigEndian>()?;

        Ok(DnsHeader {
            id,
            qr: (flags & (1 << 15)) != 0,
            opcode: ((flags >> 11) & 0xF) as u8,
            aa: (flags & (1 << 10)) != 0,
            tc: (flags & (1 << 9)) != 0,
            rd: (flags & (1 << 8)) != 0,
            ra: (flags & (1 << 7)) != 0,
            z: ((flags >> 4) & 0x7) as u8,
            rcode: (flags & 0xF) as u8,
            qdcount,
            ancount,
            nscount,
            arcount,
        })
    }
}

// Question DNS
#[derive(Debug, Clone)]
pub struct DnsQuestion {
    pub name: String,           // Nom de domaine (ex: "google.com")
    pub qtype: DnsRecordType,   // Type de requête
    pub qclass: DnsClass,       // Classe (généralement IN)
}

impl DnsQuestion {
    pub fn new(name: String, qtype: DnsRecordType) -> Self {
        DnsQuestion {
            name,
            qtype,
            qclass: DnsClass::IN,
        }
    }

    // Encoder un nom de domaine au format DNS
    // "google.com" devient [6]google[3]com[0]
    pub fn encode_name(name: &str) -> Vec<u8> {
        let mut encoded = Vec::new();

        for part in name.split('.') {
            if part.len() > 63 {
                panic!("Label trop long: {}", part);
            }
            encoded.push(part.len() as u8);
            encoded.extend_from_slice(part.as_bytes());
        }
        encoded.push(0); // Terminateur

        encoded
    }

    // Décoder un nom de domaine depuis le format DNS
    pub fn decode_name(data: &[u8], offset: &mut usize) -> IoResult<String> {
        let mut name_parts = Vec::new();
        let mut jumped = false;
        let mut jump_offset = *offset;

        loop {
            if *offset >= data.len() {
                return Err(Error::new(ErrorKind::InvalidData, "Nom tronqué"));
            }

            let length = data[*offset];

            // Compression des pointeurs (RFC 1035)
            if (length & 0xC0) == 0xC0 {
                if !jumped {
                    jump_offset = *offset + 2;
                }

                let pointer = ((length as u16 & 0x3F) << 8) | (data[*offset + 1] as u16);
                *offset = pointer as usize;
                jumped = true;
                continue;
            }

            *offset += 1;

            if length == 0 {
                break;
            }

            if *offset + length as usize > data.len() {
                return Err(Error::new(ErrorKind::InvalidData, "Label tronqué"));
            }

            let label = String::from_utf8_lossy(&data[*offset..*offset + length as usize]);
            name_parts.push(label.to_string());
            *offset += length as usize;
        }

        if jumped {
            *offset = jump_offset;
        }

        Ok(name_parts.join("."))
    }

    // Sérialiser la question DNS
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Nom encodé
        bytes.extend_from_slice(&Self::encode_name(&self.name));

        // Type (2 bytes)
        bytes.write_u16::<BigEndian>(self.qtype as u16).unwrap();

        // Classe (2 bytes)
        bytes.write_u16::<BigEndian>(self.qclass as u16).unwrap();

        bytes
    }
}

// Réponse DNS (Resource Record)
#[derive(Debug, Clone)]
pub struct DnsRecord {
    pub name: String,           // Nom de domaine
    pub rtype: DnsRecordType,   // Type d'enregistrement
    pub class: DnsClass,        // Classe
    pub ttl: u32,               // Time To Live (secondes)
    pub data: Vec<u8>,          // Données (ex: adresse IP)
}

impl DnsRecord {
    pub fn new_a_record(name: String, ip: Ipv4Addr, ttl: u32) -> Self {
        DnsRecord {
            name,
            rtype: DnsRecordType::A,
            class: DnsClass::IN,
            ttl,
            data: ip.octets().to_vec(),
        }
    }

    // Sérialiser l'enregistrement DNS
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Nom encodé
        bytes.extend_from_slice(&DnsQuestion::encode_name(&self.name));

        // Type (2 bytes)
        bytes.write_u16::<BigEndian>(self.rtype as u16).unwrap();

        // Classe (2 bytes)
        bytes.write_u16::<BigEndian>(self.class as u16).unwrap();

        // TTL (4 bytes)
        bytes.write_u32::<BigEndian>(self.ttl).unwrap();

        // RDLENGTH (2 bytes)
        bytes.write_u16::<BigEndian>(self.data.len() as u16).unwrap();

        // RDATA
        bytes.extend_from_slice(&self.data);

        bytes
    }

    // Obtenir l'adresse IP si c'est un enregistrement A
    pub fn get_ip(&self) -> Option<Ipv4Addr> {
        if self.rtype == DnsRecordType::A && self.data.len() == 4 {
            Some(Ipv4Addr::new(
                self.data[0],
                self.data[1],
                self.data[2],
                self.data[3]
            ))
        } else {
            None
        }
    }
}

// Message DNS complet
#[derive(Debug, Clone)]
pub struct DnsMessage {
    pub header: DnsHeader,
    pub questions: Vec<DnsQuestion>,
    pub answers: Vec<DnsRecord>,
    pub authorities: Vec<DnsRecord>,
    pub additionals: Vec<DnsRecord>,
}

impl DnsMessage {
    pub fn new() -> Self {
        DnsMessage {
            header: DnsHeader::new(),
            questions: Vec::new(),
            answers: Vec::new(),
            authorities: Vec::new(),
            additionals: Vec::new(),
        }
    }

    // Créer une requête DNS
    pub fn new_query(name: String, qtype: DnsRecordType) -> Self {
        let mut message = Self::new();
        message.header.qdcount = 1;
        message.questions.push(DnsQuestion::new(name, qtype));
        message
    }

    // Créer une réponse DNS
    pub fn new_response(query: &DnsMessage) -> Self {
        let mut response = query.clone();
        response.header.qr = true;  // C'est une réponse
        response.header.ra = true;  // Récursion disponible
        response.answers.clear();
        response.authorities.clear();
        response.additionals.clear();
        response
    }

    // Sérialiser le message DNS complet
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Header
        bytes.extend_from_slice(&self.header.to_bytes());

        // Questions
        for question in &self.questions {
            bytes.extend_from_slice(&question.to_bytes());
        }

        // Answers
        for answer in &self.answers {
            bytes.extend_from_slice(&answer.to_bytes());
        }

        // Authority records (pas implémenté pour la simplicité)
        // Additional records (pas implémenté pour la simplicité)

        bytes
    }

    // Désérialiser un message DNS depuis bytes
    pub fn from_bytes(data: &[u8]) -> IoResult<Self> {
        if data.len() < 12 {
            return Err(Error::new(ErrorKind::InvalidData, "Message trop court"));
        }

        let header = DnsHeader::from_bytes(data)?;
        let mut offset = 12;
        let mut questions = Vec::new();
        let mut answers = Vec::new();

        // Parser les questions
        for _ in 0..header.qdcount {
            if offset >= data.len() {
                return Err(Error::new(ErrorKind::InvalidData, "Question tronquée"));
            }

            let name = DnsQuestion::decode_name(data, &mut offset)?;

            if offset + 4 > data.len() {
                return Err(Error::new(ErrorKind::InvalidData, "Question incomplète"));
            }

            let mut cursor = Cursor::new(&data[offset..]);
            let qtype_num = cursor.read_u16::<BigEndian>()?;
            let qclass_num = cursor.read_u16::<BigEndian>()?;
            offset += 4;

            let qtype = DnsRecordType::from_u16(qtype_num)
                .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Type de question invalide"))?;
            let qclass = DnsClass::from_u16(qclass_num)
                .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Classe de question invalide"))?;

            questions.push(DnsQuestion {
                name,
                qtype,
                qclass,
            });
        }

        // Parser les réponses (version simplifiée)
        for _ in 0..header.ancount {
            if offset >= data.len() {
                break;
            }

            let name = DnsQuestion::decode_name(data, &mut offset)?;

            if offset + 10 > data.len() {
                break;
            }

            let mut cursor = Cursor::new(&data[offset..]);
            let rtype_num = cursor.read_u16::<BigEndian>()?;
            let class_num = cursor.read_u16::<BigEndian>()?;
            let ttl = cursor.read_u32::<BigEndian>()?;
            let rdlength = cursor.read_u16::<BigEndian>()?;
            offset += 10;

            if offset + rdlength as usize > data.len() {
                break;
            }

            let rdata = data[offset..offset + rdlength as usize].to_vec();
            offset += rdlength as usize;

            if let (Some(rtype), Some(class)) =
                (DnsRecordType::from_u16(rtype_num), DnsClass::from_u16(class_num)) {
                answers.push(DnsRecord {
                    name,
                    rtype,
                    class,
                    ttl,
                    data: rdata,
                });
            }
        }

        Ok(DnsMessage {
            header,
            questions,
            answers,
            authorities: Vec::new(),
            additionals: Vec::new(),
        })
    }
}

// Base de données DNS simple pour le serveur
#[derive(Debug, Clone)]
pub struct SimpleDnsDatabase {
    records: HashMap<String, Ipv4Addr>,
}

impl SimpleDnsDatabase {
    pub fn new() -> Self {
        let mut db = SimpleDnsDatabase {
            records: HashMap::new(),
        };

        // Ajouter quelques enregistrements prédéfinis
        db.add_record("localhost".to_string(), "127.0.0.1".parse().unwrap());
        db.add_record("test.local".to_string(), "192.168.1.100".parse().unwrap());
        db.add_record("server.local".to_string(), "192.168.1.1".parse().unwrap());
        db.add_record("example.com".to_string(), "93.184.216.34".parse().unwrap());
        db.add_record("google.com".to_string(), "8.8.8.8".parse().unwrap());

        db
    }

    pub fn add_record(&mut self, name: String, ip: Ipv4Addr) {
        self.records.insert(name.to_lowercase(), ip);
    }

    pub fn lookup(&self, name: &str) -> Option<Ipv4Addr> {
        self.records.get(&name.to_lowercase()).copied()
    }

    pub fn list_records(&self) -> &HashMap<String, Ipv4Addr> {
        &self.records
    }
}