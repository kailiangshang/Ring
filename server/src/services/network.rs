use crate::error::{Result, RingError};

pub fn get_local_ip() -> Result<String> {
    // Try to determine local IP by connecting to a public DNS
    use std::net::{SocketAddr, TcpStream, ToSocketAddrs};

    let addrs: Vec<SocketAddr> = "8.8.8.8:80"
        .to_socket_addrs()
        .map_err(|e| RingError::Internal(format!("failed to resolve address: {e}")))?
        .collect();

    if addrs.is_empty() {
        return Err(RingError::Internal("no address resolved".into()));
    }

    let stream = TcpStream::connect(addrs[0])
        .map_err(|e| RingError::Internal(format!("failed to connect: {e}")))?;

    let local_addr = stream
        .local_addr()
        .map_err(|e| RingError::Internal(format!("failed to get local addr: {e}")))?;

    Ok(local_addr.ip().to_string())
}
