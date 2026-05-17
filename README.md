## Experiment 2.1
![alt text](assets/experiment2.1.png)
Cara Menjalankan:

Buka satu terminal, lalu jalankan server dengan perintah: cargo run --bin server. Server akan berjalan dan mendengarkan pada port 2000.

Buka terminal baru (bisa di-split atau di tab baru), dan jalankan client pertama dengan perintah: cargo run --bin client.

Buka lagi dua terminal baru, dan jalankan client kedua dan ketiga menggunakan perintah yang sama.

Apa yang Terjadi Ketika Mengetik Pesan?
Ketika klien pertama mengetik sesuatu dan menekan Enter, input tersebut ditangkap oleh fungsi stdin.next_line() dan dikirim ke server melalui protokol WebSocket (ws_stream.send(...)).

Server menggunakan makro tokio::select! untuk menangani banyak event secara asinkron. Ketika server menerima pesan, ia mengirimkannya ke bcast_tx (Tokio Broadcast Channel). Channel ini lalu "menyiarkan" (broadcast) pesan tersebut ke semua fungsi receiver (bcast_rx) yang dimiliki klien lain, sehingga pesan tersebut muncul di layar terminal milik klien kedua dan ketiga di waktu yang hampir bersamaan.

## Experiment 2.2
![alt text](assets/experiment2.2.png)
Penjelasan Modifikasi Port:
Pada eksperimen ini, port koneksi diubah dari 2000 menjadi 8080. Karena komunikasi ini bersifat client-server, perubahan port wajib dilakukan di kedua sisi:  

Di sisi Server: TcpListener::bind("127.0.0.1:8080") agar server mendengarkan permintaan masuk pada port 8080.

Di sisi Client: Uri::from_static("ws://127.0.0.1:8080") agar klien mencoba menyambung ke port yang tepat.

Program ini tetap menggunakan protokol WebSocket yang sama.

Pada sisi Client, protokol ini didefinisikan secara eksplisit pada awalan (scheme) URI koneksinya, yaitu awalan ws:// pada string "ws://127.0.0.1:8080".

Pada sisi Server, protokol ini tidak didefinisikan melalui string URL, melainkan diterapkan dengan cara menerima koneksi Transmission Control Protocol (TCP) biasa, lalu melakukan upgrade koneksi tersebut menjadi jalur WebSocket menggunakan fungsi ServerBuilder::new().accept(socket) dari library tokio-websockets.