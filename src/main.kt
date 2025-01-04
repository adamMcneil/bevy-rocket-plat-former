import java.io.IOException; 
import java.net.DatagramPacket; 
import java.net.DatagramSocket; 
import java.net.InetAddress; 
import java.util.Scanner; 
import java.nio.charset.Charset

fun main() {
  sendPacket("10.0.0.14", "8000", "something")
    // var ds = DatagramSocket()
    //
    //
    // var ip = InetAddress.getByName("127.0.0.1")
    //
    // val buf = byteArrayOf(0x48, 101, 108, 108, 111)
    //
    // var DpSend = DatagramPacket(buf, 5, ip, 34254); 
    // for (i in 0..3) {
    //     ds.send(DpSend); 
    //     println("sent packet")
    // }
}

fun sendPacket(ip: String, port: String, msg: String) {
    val ds = DatagramSocket()
    val ipAddress = InetAddress.getByName(ip)
    val buffer = msg.toByteArray(Charset.defaultCharset())
    val dpSend = DatagramPacket(buffer, buffer.size, ipAddress, port.toInt());
    ds.send(dpSend);
}
