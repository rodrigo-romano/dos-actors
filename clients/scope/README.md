# GMT DOS Actors Scope

`gmt_dos-clients_scope` acquire signals from a transmitter and display them graphically.

The communication between the transmitter and the scope is secured with a signed certificate
that must be provided by the transmitter.

## AWS EC2 Instance Setup

To stream data to a local scope from an AWS EC2 instance, a new inboud rule needs to be added to the Secury Group of the instance, a rule with the UDP protocol, a port or a port range, and any IPv4 source selected, for example:
![Alt text](aws-ec2-udp-settings.png)


AWS EC2 instances have 2 IPs, a local or private IP and a public IP.
The IPs can be found from the AWS dashboard or from a terminal connected to the instance by running: `ec2metadata | grep ip`.

To check that the new rule of the Security Group is setup properly, on the instance listen to one of the UDP port with
```shell
netcat -luv 5001
```
and on the local machine send a message to the instance UDP port with 
```shell
echo "hello world" | netcat -uv <instance-public-ip> 5001
```
On the instance, the following should be written at the prompt:
```shell
Connection from <your-machine-ip> <your-machine-port> received!
hello world
```

The instance local IP is assigned to the transmitter defined on the instance e.g.:
```rust
let tx = Transceiver::<U>::transmitter(instance_private_ip)?;
```
and the instance public IP is assigned to the scope defined in the local application e.g.
```rust
Scope::new(instance_public_ip, "0.0.0.0:0")
    .signal::<U>(delta_t)?
    .show();
```