// Copyright (c) 2016-2017 Chef Software Inc. and/or applicable contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import { Action } from "@ngrx/store";
import { addNotification, SUCCESS, DANGER } from "../actions/notifications";
import { requestRoute } from "../actions/router";
import * as depotApi from "../depotApi";
import { BuilderApiClient } from "../BuilderApiClient";
import { parseKey } from "../util";
import {
    fetchIntegrations
} from "./origin-page/origin-integrations-tab/origin-integrations-tab.actions";
import { Origin } from "./origin.model";

export const POPULATE_MY_ORIGINS = "POPULATE_MY_ORIGINS";
export const POPULATE_ORIGIN_INVITATIONS = "POPULATE_ORIGIN_INVITATIONS";
export const SET_CURRENT_ORIGIN = "SET_CURRENT_ORIGIN";
export const SET_CURRENT_ORIGIN_LOADING = "SET_CURRENT_ORIGIN_LOADING";
export const SET_CURRENT_ORIGIN_ADDING_PRIVATE_KEY =
    "SET_CURRENT_ORIGIN_ADDING_PRIVATE_KEY";
export const SET_CURRENT_ORIGIN_ADDING_PUBLIC_KEY =
    "SET_CURRENT_ORIGIN_ADDING_PUBLIC_KEY";
export const SET_ORIGIN_INTEGRATION_SAVE_ERROR_MESSAGE =
    "SET_ORIGIN_INTEGRATION_SAVE_ERROR_MESSAGE";
export const TOGGLE_ORIGIN_PICKER = "TOGGLE_ORIGIN_PICKER";
export const SET_PACKAGE_COUNT_FOR_ORIGIN = "SET_PACKAGE_COUNT_FOR_ORIGIN";
export const SET_ORIGIN_PRIVACY_SETTINGS = "SET_ORIGIN_PRIVACY_SETTINGS";

export class ToggleOriginPicker implements Action {
    readonly type = TOGGLE_ORIGIN_PICKER;
}

export class PopulatePackageCountForOrigin implements Action {
    readonly type = SET_PACKAGE_COUNT_FOR_ORIGIN;
    constructor(public payload: number) {}
}

export class PopulateMyOrigins implements Action {
    readonly type = POPULATE_MY_ORIGINS;
    constructor(public payload: Origin[], public error: string) {}
}

export class PopulateOriginInvitations implements Action {
    readonly type = POPULATE_ORIGIN_INVITATIONS;
    constructor(public payload: Origin[], public error: string) {}
}

export class SetCurrentOrigin implements Action {
    readonly type = SET_CURRENT_ORIGIN;
    constructor(public payload: Origin[], public error: string) {}
}

export class SetCurrentOriginLoading implements Action {
    readonly type = SET_CURRENT_ORIGIN_LOADING;
    constructor(public payload: Origin[]) {}
}

export class SetOriginIntegrationSaveErrorMessage implements Action {
    readonly type = SET_ORIGIN_INTEGRATION_SAVE_ERROR_MESSAGE;
    constructor(public payload: Origin[]) {}
}
