use super::Device;
use crate::devices::Patch;
use crate::patch::PatchFile;
use crate::server::SettingsRequest;
use crate::utils;

pub struct DeviceGeneric {
    max_tdp: i8,
}

impl DeviceGeneric {
    pub fn new(max_tdp: i8) -> DeviceGeneric {
        DeviceGeneric { max_tdp }
    }
}

impl Device for DeviceGeneric {
    fn update_settings(&self, request: SettingsRequest) {
        if let Some(per_app) = &request.per_app {
            // TDP changes
            if let Some(tdp) = per_app.tdp_limit {
                self.set_tdp(tdp);
            }
        }
    }

    fn set_tdp(&self, tdp: i8) {
        // Update TDP
        let target_tdp = tdp as i32 * 1;

        let command = [
            "intel_set_prefs",
            &format!("cpu_max_perf {}", target_tdp),
        ];
        match utils::run_command(&command) {
            Ok(_) => println!("Set TDP successfully!"),
            Err(_) => println!("Couldn't set TDP"),
        }
    }

    fn get_patches(&self) -> Vec<Patch> {
        vec![
            // Max TDP = 28
            Patch {
                text_to_find: "return[n,t,r,e=>i((()=>p.Get().SetTDPLimit(e)))".to_string(),
                replacement_text: format!("return[n,t,{:?},e=>i((()=>p.Get().SetTDPLimit(e)))", self.max_tdp).to_string(),
                destination: PatchFile::Chunk,
            },
            // Listen to TDP changes
            Patch {
                text_to_find: "const t=c.Hm.deserializeBinary(e).toObject();Object.keys(t)".to_string(),
                replacement_text: "const t=c.Hm.deserializeBinary(e).toObject(); console.log(t); fetch(`http://localhost:1338/update_settings`, { method: 'POST',  headers: {'Content-Type': 'application/json'}, body: JSON.stringify(t.settings)}); Object.keys(t)".to_string(),
                destination: PatchFile::Chunk,
            },
            // Replace Xbox menu button with Steam one
            Patch {
                text_to_find: "case 4:return l.createElement".to_string(),
                replacement_text: "case 4:case 31: return l.createElement".to_string(),
                destination: PatchFile::Chunk,
            },

            // Change resolution to Native (if Default) after installation
            Patch {
                text_to_find: "DownloadComplete_Title\"),o=ze(r,t.data.appid());const s=(0,O.Q2)();".to_string(),
                replacement_text: "DownloadComplete_Title\"),o=ze(r,t.data.appid()); SteamClient.Apps.GetResolutionOverrideForApp(t.data.appid()).then(res => res === \"Default\" && SteamClient.Apps.SetAppResolutionOverride(t.data.appid(), \"Native\")); const s=(0,O.Q2)();".to_string(),
                destination: PatchFile::Chunk,
            },
        ]
    }

    fn get_key_mapper(&self) -> Option<tokio::task::JoinHandle<()>> {
        None
    }
}
