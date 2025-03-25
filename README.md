# 🚀 Sessionizer: A Tmux Sessionizer in Rust! 🔥

Welcome to **Sessionizer** – your new best friend for managing tmux sessions! If you're all about a streamlined workflow without the fuss of complicated bash scripts, you're in the right place. Let's elevate your terminal game with some Rust-powered magic! 😎💻

## ✨ What is Sessionizer?

Sessionizer is a lightweight, high-performance tmux session manager written in Rust. It’s designed to help you effortlessly organize, switch, and manage your tmux sessions so you can focus on what really matters – getting things done! 🚀💡

## 💡 Features

- **Lightning Fast Performance** ⚡  
  Built with Rust, Sessionizer is speedy and efficient, ensuring your sessions are managed in a snap.

- **User-Friendly Interface** 👍  
  you already know fzf, so you will rock this!

- **ssh sessions can be configured in a toml file!** 👍  
  ok here is actually the main feature - you can configure a session like this:

  ```toml
  [[sessions]]
  name = "awesomeremotesessionname"
  protocol = "ssh"
  host = "root@host"
  remote_command = "cd foo/bar; nvim -c \"Telescope find_files\""
  split = { type = "hs", command = "docker exec -it somecontainerontheremotehost tail -f /var/log/apache2/*.log" }
  ```

  this will create a new session to the remote host, create a horizontal split (hs) and in that horizontal split it will run the defined command

- **User-Friendly Interface** 👍  
  
- **Minimalistic & Focused** 🎯  
  Designed to do one thing well: manage your tmux sessions with ease.

- **Open Source Goodness** 🤝  
  Your ideas and contributions can help make this tool even better – let’s build something awesome together!

## 📦 Installation

Getting started is as easy as pie! Follow these steps:

1. **Clone the Repository:**

   ```bash
   git clone https://github.com/schneipp/sessionizer.git
   cd sessionizer
2. **Compile:**

   ```bash
   cargo build --release
   ```

3. **Copy the Binary to Your PATH:**

  ```bash
  cp target/release/sessionizer /usr/local/bin/
```

🛠️ Usage
Run Sessionizer with a simple command:

```bash
sessionizer
```

Follow the on-screen instructions to create, list, and switch between tmux sessions. It’s as easy as 1-2-3! 💯

🎯 Why Sessionizer?
I built Sessionizer to bring a touch of positivity and efficiency to everyday workflows. I am very bad ad bash scripting, so hacking in the primeagens sessionizer script around wasn't ideal. No more messy scripts—just a clean, simple, and super motivating tool to help you conquer your tasks. Embrace the joy of organized sessions and let your productivity soar!

🤗 Contributing
Contributions are welcome and celebrated! If you have ideas, improvements, or bug fixes, please open an issue or submit a pull request. Let's collaborate and make Sessionizer even more amazing together! 🌟

📄 License
Sessionizer is released under the BSD License. Feel free to use, modify, and share it as you see fit. You can even create a crypto coin from the code and make zeroes of dollars 📝

🙌 Stay Connected
If you love what you see, consider starring this project on GitHub ⭐. Your feedback, questions, or even a friendly hello are always welcome. Happy coding and keep sessionizing! 🚀💻✨

Made with neovim, by schneipp
