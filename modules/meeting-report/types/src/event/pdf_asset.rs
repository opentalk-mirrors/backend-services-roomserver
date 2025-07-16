// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::assets::AssetId;
use serde::{Deserialize, Serialize};

/// Handle to a PDF asset
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PdfAsset {
    /// The file name of the PDF asset
    pub filename: String,

    /// The asset id for the PDF asset
    pub asset_id: AssetId,
}
