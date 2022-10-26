(* Validator registration contract

   The owner of the contract has control over the registrations, especially in Genesis and Sealed states,
   where it is the only one who can register new validators. In Open state validators have to register themselves.

   Validators need to be Tezos bakers with >6000tez voting power, they are identified by their key_hash.

   Updating a registration or removing one is allowed. A history of old registrations is kept in the storage.

   It's been deployed on Ghostnet as KT1WfuYfNmZVgvEgdeBM7KL73WPneiEbQT83
*)


(* 6,000tez in mutez as a nat; for checking the VOTING_POWER *)
let required_voting_power : nat = 1_000n

(* Allowed actions depend on the state the contract is in, mainly important for safe bootstrap of the subnetwork *)
type state =
    Genesis
  | Sealed
  | Open

(* To be used on the subnetwork for safe communication *)
type tls_certificate = bytes

(* This type is used as a key in maps throughout the contract *)
type validator = key_hash

(* Registration data *)
type register = {
  baking_account : validator;
  (* subnetwork specific public key *)
  public_key : key;
  (* Hash of the TLS public key certificate used to connect to peers on `subzero` *)
  tls_cert : tls_certificate;
  (* Subnetwork threshold key *)
  (* threshold_public_key : key option; *)
}

(* Registration history data *)
type register_history = {
  (* Registration data *)
  register : register;
  (* timestamp of the event *)
  timestamp : timestamp
}

(* The timestamp shows when the registration got outdated (unregistered or updated) *)
type validator_history = (register_history) list
type history_map = (validator, validator_history) big_map

type storage = {
  (* The current state of the registration procedure *)
  state : state;
  (* Owner of the contract *)
  owner : address;
  (* TODO not sure if we'll need this, if we can intercept all registrations *)
  (*Set of registered validators *)
  validators : validator set;
  (* big_map containing the registered information for each validator *)
  validator_map : (validator, register) big_map;
  (* validators with change history of registration, this might blow up the state *)
  old_validators : validator set;
  (* big_map for keeping history of validator registrations, this might blow up the state *)
  old_validator_map: history_map;
}

type return = operation list * storage


let check_owner (owner : address) =
  if not (owner = Tezos.get_sender ()) then failwith "Unauthorised: only owner can call this"

let check_validator_sender (baker : validator) =
  let sender = Tezos.get_sender () in
  let contr = Tezos.implicit_account baker in
  let addr = Tezos.address contr in
  if addr <> sender then
    failwith "A baker needs to register itself"

let store_history (reg : register) (old_validators : validator set) (old_map : history_map)
    : validator set * history_map =
  let validator = reg.baking_account in
  let now = Tezos.get_now () in
  let history : register_history = { register = reg; timestamp = now } in
  let map = match Big_map.find_opt reg.baking_account old_map with
      None -> Big_map.add validator [history] old_map
    | Some list -> Big_map.add validator (history :: list) old_map in
  Set.add validator old_validators, map


let do_register (reg : register) (store : storage) =
  let key = reg.baking_account in
  let () = if Tezos.voting_power key < required_voting_power
    then failwith "Baker doesn't have enough voting power" in
  let (old_validators, old_map) = if Set.mem key store.validators then
      let old_reg = Option.unopt (Big_map.find_opt key store.validator_map) in
      store_history old_reg store.old_validators store.old_validator_map
    else store.old_validators, store.old_validator_map in
  let set = Set.add key store.validators in
  let map = Big_map.add key reg store.validator_map in
  let store = { store with validators = set; validator_map = map; old_validator_map = old_map;
                old_validators = old_validators } in
  ([] : operation list), store

let do_unregister (validator : validator) (store : storage) =
  let set = if Set.mem validator store.validators then
      Set.remove validator store.validators
    else failwith "Validator not registered" in
  let (old_validators, old_map) = if Set.mem validator store.validators then
      let old_reg = Option.unopt (Big_map.find_opt validator store.validator_map) in
      store_history old_reg store.old_validators store.old_validator_map
    else store.old_validators, store.old_validator_map in
   (* TODO do we want to remove it from the map? *)
  let map = Big_map.remove validator store.validator_map in
  let store = { store with validators = set; validator_map = map; old_validator_map = old_map;
                old_validators = old_validators } in
  ([] : operation list), store


(* Entrypoints *)

let register_validator (reg, store : register * storage) : return =
  match store.state with
      Genesis -> (failwith "Cannot register validators during genesis" : operation list), store
    | Sealed ->
       let () = check_owner store.owner in
       do_register reg store
    | Open ->
        let () = check_validator_sender reg.baking_account in
        do_register reg store

let unregister_validator (validator, store : validator * storage) : return =
  match store.state with
      Genesis -> (failwith "Cannot unregister validators during genesis" : operation list), store
    | Sealed ->
       let () = check_owner store.owner in
       do_unregister validator store
    | Open ->
        (* The owner is allowed to remove validators *)
        let () = if not (store.owner = Tezos.get_sender ())
          then check_validator_sender validator in
        do_unregister validator store

let change_state (state, store : state * storage) : return =
  let () = check_owner store.owner in
  let store = { store with state = state } in
  ([] : operation list), store


type parameter =
    Register of register
  | State_change of state
  | Unregister of validator

let main (action, store : parameter * storage) : return =
  match action with
      Register reg -> register_validator (reg, store)
    | State_change state -> change_state (state, store)
    | Unregister validator -> unregister_validator (validator, store)
