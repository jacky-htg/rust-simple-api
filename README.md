# CRUD API

`crud-api` adalah sebuah aplikasi API sederhana yang dibangun dengan Rust menggunakan tokio untuk pengolahan asynchronous. API ini menyediakan operasi CRUD (Create, Read, Update, Delete) untuk manajemen pengguna dan dilengkapi dengan autentikasi.

## Daftar Isi

- [Fitur](#fitur)
- [Prasyarat](#prasyarat)
- [Instalasi](#instalasi)
- [Penggunaan](#penggunaan)
- [Struktur Proyek](#struktur-proyek)
- [Kontribusi](#kontribusi)
- [Lisensi](#lisensi)

## Fitur

1. Asynchronous Programming
Aplikasi ini dibangun menggunakan Rust dengan asynchronous programming melalui crate tokio. Ini memungkinkan aplikasi untuk menangani banyak permintaan secara bersamaan tanpa memblokir thread. Dengan menggunakan async dan await, Anda dapat menulis kode yang lebih efisien dan responsif, yang sangat penting untuk aplikasi jaringan dan API yang harus menangani banyak koneksi sekaligus.

2. Concurrency Limits
Fitur concurrency limits diterapkan untuk mengontrol jumlah koneksi yang dapat diproses secara bersamaan. Dalam proyek ini, tokio::sync::Semaphore digunakan untuk membatasi jumlah permintaan yang dapat diproses secara bersamaan oleh server. Ini membantu mencegah beban berlebih pada server dan memastikan bahwa sumber daya yang tersedia digunakan secara efisien.

3. Rate Limiting
Rate limiting digunakan untuk mengontrol jumlah permintaan yang dapat dilakukan oleh pengguna dalam periode waktu tertentu. Dengan menggunakan crate governor, API ini menerapkan batasan pada jumlah permintaan yang dapat diterima dalam interval tertentu. Hal ini penting untuk melindungi API dari potensi serangan DoS (Denial of Service) dengan membatasi jumlah permintaan yang dapat diajukan oleh klien dalam waktu singkat.

4. Handling Pool Connection untuk Database
Aplikasi ini menggunakan connection pooling untuk manajemen koneksi ke database PostgreSQL dengan menggunakan crate deadpool-postgres. Connection pool memungkinkan aplikasi untuk memelihara beberapa koneksi database dan mengelolanya secara efisien. Alih-alih membuka dan menutup koneksi setiap kali permintaan baru datang, aplikasi dapat menggunakan koneksi yang sudah ada dalam pool, yang mengurangi latensi dan meningkatkan performa.

5. Dependency Injection Pattern
Dependency injection digunakan untuk memisahkan komponen dalam aplikasi dan meningkatkan modularitas serta testabilitas. Dalam proyek ini, struktur AppState menyimpan pool database dan limiter, yang diinject ke dalam handler untuk digunakan dalam proses permintaan. Ini memungkinkan Anda untuk mengelola dan mengganti dependensi dengan lebih mudah, serta melakukan pengujian unit pada setiap komponen secara terpisah.

6. Logger
Aplikasi ini menggunakan crate log dan env_logger untuk mencatat log dalam aplikasi. Logger diatur untuk mencetak informasi log dengan tingkat detail yang berbeda (info, debug, error) dan dapat dikonfigurasi menggunakan variabel lingkungan (RUST_LOG). Logging yang baik membantu dalam pemantauan aplikasi dan memudahkan debugging ketika terjadi kesalahan.

7. Graceful Shutdown
Fitur graceful shutdown memungkinkan aplikasi untuk menangani permintaan yang sedang berlangsung dan menutup koneksi dengan baik saat aplikasi dihentikan. Dalam proyek ini, aplikasi menunggu sinyal ctrl_c untuk memicu proses shutdown. Ketika sinyal diterima, server berhenti menerima koneksi baru dan menyelesaikan semua koneksi yang sedang aktif sebelum benar-benar keluar. Ini membantu mencegah kehilangan data atau permintaan yang tidak selesai.

8. Handling CORS (Cross-Origin Resource Sharing)
CORS adalah mekanisme yang memungkinkan aplikasi web di satu domain untuk mengakses sumber daya di domain lain. Dalam aplikasi ini, ketika menerima permintaan OPTIONS, server mengembalikan header CORS yang sesuai untuk mengizinkan akses dari domain lain. Hal ini penting untuk aplikasi yang berinteraksi dengan frontend yang mungkin berjalan di domain berbeda.

9. JSON Web Token (JWT)
API ini menggunakan JWT untuk autentikasi pengguna. Setelah pengguna berhasil login, server mengeluarkan token JWT yang berisi informasi yang diperlukan untuk mengautentikasi permintaan selanjutnya. Token ini dikirim kembali ke klien dan harus disertakan dalam header permintaan yang membutuhkan autentikasi. Dengan menggunakan JWT, aplikasi dapat memastikan bahwa hanya pengguna yang terautentikasi yang dapat mengakses endpoint tertentu, meningkatkan keamanan aplikasi secara keseluruhan.

10. CRUD Users
API ini menyediakan endpoint untuk mengelola modul users, meliputi :
- Menambahkan pengguna baru.
- Mengambil detail pengguna berdasarkan ID.
- Mengambil daftar semua pengguna.
- Memperbarui informasi pengguna.
- Menghapus pengguna.

## Prasyarat

Sebelum menjalankan proyek ini, pastikan Anda memiliki hal-hal berikut:

- [Rust](https://www.rust-lang.org/tools/install) versi terbaru.
- PostgreSQL terinstal dan berjalan.
- Cargo untuk manajemen paket Rust.

## Instalasi

1. Clone repositori ini:

   ```bash
   git clone https://github.com/username/crud-api.git
   cd crud-api
   ```

2. Buat variabel env di OS anda:

```bash
export DATABASE_URL=postgres://username:password@localhost/db_name
export SECRET_KEY=rahasia
```
Gantilah username, password, dan db_name sesuai dengan konfigurasi PostgreSQL Anda. Juga ganti SECRET_KEY dengan key anda sendiri. 

3. Jalankan perintah berikut untuk membangun dan menjalankan aplikasi:

```bash
RUST_LOG=debug cargo run
```

## Penggunaan
Setelah server berjalan, Anda dapat mengakses API pada http://localhost:8080. Berikut adalah beberapa contoh permintaan yang dapat Anda coba:

### Menambahkan Pengguna
```http
POST /users
Content-Type: application/json

{
    "email": "user@example.com",
    "password": "password123"
}
```

### Mengambil Daftar Pengguna
```http
GET /users
Mengambil Detail Pengguna
```

```http
GET /users/{id}
```

### Memperbarui Pengguna

```http
PUT /users/{id}
Content-Type: application/json

{
    "email": "new_email@example.com"
}
```

### Menghapus Pengguna
```http
DELETE /users/{id}
```

### Autentikasi Pengguna
```http
POST /login
Content-Type: application/json

{
    "email": "user@example.com",
    "password": "password123"
}
```

## Struktur Proyek
```bash
crud-api
├── Cargo.toml
├── src
│   ├── auth
│   │   └── handler.rs         # Handler untuk autentikasi
│   ├── libs
│   │   └── mod.rs              # Fungsi utilitas umum
│   ├── users
│   │   └── handler.rs          # Handler untuk operasi pengguna
│   └── main.rs                 # Titik masuk aplikasi
└── .env                        # Konfigurasi variabel lingkungan
```

## Kontribusi
Kontribusi sangat diterima! Silakan buka issue atau buat pull request jika Anda ingin berkontribusi pada proyek ini.

## Lisensi
Proyek ini dilisensikan di bawah GNU GPL License.