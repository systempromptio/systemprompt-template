# DigitalOcean Marketplace 1-Click image for the systemprompt gateway.
# Build: packer init . && packer build -var "do_token=$DO_API_TOKEN" marketplace-image.pkr.hcl
# Conventions per github.com/digitalocean/marketplace-partners.
packer {
  required_plugins {
    digitalocean = {
      version = ">= 1.0.4"
      source  = "github.com/digitalocean/digitalocean"
    }
  }
}

variable "do_token" {
  type      = string
  sensitive = true
}

variable "image_version" {
  type    = string
  default = "0.4.0"
}

source "digitalocean" "systemprompt" {
  api_token     = var.do_token
  image         = "ubuntu-24-04-x64"
  region        = "nyc3"
  size          = "s-2vcpu-4gb"
  ssh_username  = "root"
  snapshot_name = "systemprompt-${var.image_version}-{{timestamp}}"
}

build {
  sources = ["source.digitalocean.systemprompt"]

  provisioner "file" {
    source      = "files/"
    destination = "/"
  }

  provisioner "shell" {
    scripts = [
      "scripts/010-docker.sh",
      "scripts/020-systemprompt.sh",
      "scripts/900-cleanup.sh",
    ]
    environment_vars = ["IMAGE_VERSION=${var.image_version}"]
  }
}
