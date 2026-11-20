// Two identities for a test's own site over https: each a key and a certificate that signs
// itself, for 127.0.0.1, which no device trusts. For the tests of what an engine is told, and
// tells, when a certificate cannot be trusted, and of going on past one (021, 026).
//
// They are kept here, as PKCS#12 in base64, because Apple's systems make a TLS identity in
// memory from nothing else: a key made at test time can only become an identity by way of a
// keychain, and a test never touches the person's keychain. THESE KEYS PROTECT NOTHING: they
// are public, in a public repository, and name only this machine's loopback address. Made with:
//
//   openssl ecparam -genkey -name prime256v1 -noout -out first.key
//   openssl req -x509 -new -key first.key -days 7300 -sha256 -config san.cnf -out first.pem
//   openssl pkcs12 -export -inkey first.key -in first.pem -out first.p12 -passout pass:webtiger-test
//
// where san.cnf gives CN=127.0.0.1 and subjectAltName = IP:127.0.0.1, DNS:localhost.
import Foundation
import Security

enum TestIdentity: CaseIterable, Sendable {
    case first
    /// Another key and certificate for the same place: a host whose certificate changes.
    case second

    private static let passphrase = "webtiger-test"

    private var encoded: String {
        switch self {
        case .first: Self.firstEncoded
        case .second: Self.secondEncoded
        }
    }

    /// The identity, made in this process's memory and nowhere else: `kSecImportToMemoryOnly`
    /// keeps it out of every keychain, on a Mac too. `nil` when the system will not make it.
    func make() -> SecIdentity? {
        guard let data = Data(base64Encoded: encoded) else { return nil }
        let options: [CFString: Any] = [kSecImportExportPassphrase: Self.passphrase, kSecImportToMemoryOnly: true]
        var items: CFArray?
        guard SecPKCS12Import(data as CFData, options as CFDictionary, &items) == errSecSuccess,
              let first = (items as? [[String: Any]])?.first,
              let identity = first[kSecImportItemIdentity as String]
        else { return nil }
        // The dictionary's value is the identity: Core Foundation's own cast.
        return (identity as! SecIdentity) // swiftlint:disable:this force_cast
    }

    private static let firstEncoded = [
        "MIIEFQIBAzCCA9sGCSqGSIb3DQEHAaCCA8wEggPIMIIDxDCCAn8GCSqGSIb3DQEHBqCCAnAwggJsAgEAMIICZQYJKoZIhvcNAQcB",
        "MBwGCiqGSIb3DQEMAQYwDgQI99vYLGNyDRICAggAgIICOI3XGQxKe1YCcGmQms6IzlsNl8+lS44PR2Vn60RLXiEIhtiMNQGmnHtq",
        "qGaCyKEEvCzOUmd8t5esLslnXt0vjPMr/nYTBjISBwVgk9/QiwcmiDVXfQ8fhryZSlnUdTRC54/Pvz1BDoknVzNWy3OWOx+unrAj",
        "RHw4+i1FkEnmEVK/Zd0ILp6Ho2VawVk8OasDr5/iGZOVe4gYCYf2k9cNyu1N0mIJDRUH01x/xsK51FXRoz9h9u+7ZvpEmERzNBsJ",
        "i6GPu9ZCPkqzaiLIhIX+pU8LHnxv2LRFnVIWzx4QGszoICu26dcKKWBrT6JmsHCBQR4wDMPBBj8RKFskWNiWYjW5s940BVD8t7LQ",
        "IA+zN3l6NPJuAbYs59wHBXngeDyvcycOs4nrm2cW3D2uUjDlx614PweQx5qx1luv3iobTuJB9Hy1eVaEsnI1OXTts+lAqwDkUzDS",
        "E72NB8eLIvS9/O0jHKMtaNvNUPC5BfECz2v7ySzDIUGba8FleBUrdPffZAZyZHQMAW7y6JZe1Olwwq8AZfhjv5WRrlxmjPODkQel",
        "6ZYamq3ZOHjdtZbDNX90AO1d0pahAmGQKwNtsZ2+3rDAY6sUMJqnN8W9VQ2NPUjgzO4Fe/dmhrzekYvOm3y/lWI3jFOv+zO3zn0u",
        "R53NR18r2PJDWt6AW1pN52PU2mI29x4bRXflUYSf3Bip0Iv3j4fjnooNSaVUHe6d6sANENZC6HKj7uq7oQRjPrKt10bewVi8cRD1",
        "kBUwggE9BgkqhkiG9w0BBwGgggEuBIIBKjCCASYwggEiBgsqhkiG9w0BDAoBAqCBtDCBsTAcBgoqhkiG9w0BDAEDMA4ECGFj05Ln",
        "otCcAgIIAASBkP7uABDJTw4FTsIGlWIW7qyXTipPrLJA2sVBF3dof9bN52d1sXqjO5wKMX98CCYv7e7CZ1ZsFx3fPS+xT2ld2pZZ",
        "TVFR0+c/bpM1gz7V7aU7SlaU0/g1Yg/x5nzG1chgy89jieI0QDEEaR+NnLK/11/6oeJxkpKp5ZUxn7NVW/TYsFNjYQ9HWFYxXyHF",
        "QiK4MzFcMCMGCSqGSIb3DQEJFTEWBBQz0blv+Y6BhPFUUuxvvQ+q7S1mCzA1BgkqhkiG9w0BCRQxKB4mAHcAZQBiAHQAaQBnAGUA",
        "cgAtAHQAZQBzAHQALQBmAGkAcgBzAHQwMTAhMAkGBSsOAwIaBQAEFGGO/u58aI9U9zKJLWW6tFHq2EojBAhtd5bz7enuuAICCAA=",
    ].joined()

    private static let secondEncoded = [
        "MIIEFwIBAzCCA90GCSqGSIb3DQEHAaCCA84EggPKMIIDxjCCAn8GCSqGSIb3DQEHBqCCAnAwggJsAgEAMIICZQYJKoZIhvcNAQcB",
        "MBwGCiqGSIb3DQEMAQYwDgQIKTVHKpEqX8ECAggAgIICOKPDUTcBq/U4Gx3qbnhAfFyPkucyRtvq1ouVCvyf0JWHxNjimdWPK6jF",
        "No73ZscGvmMrteIbEVa0AS+94mWDTOlloiuN4XUvZjpUx2NZ5eWh+N4VND7wc693UR3sk0xUxAZe5X8dkPsJkD0/YtSzjBqhRvi0",
        "xkUy1TYG2k4n4U0z+fD2fVjg3s+LtGlVrwZqi9MSiFgmXGCmsHtoOJw1fiCg5mlwK8yRPs7ZGJF/xZczWjvMpagq1WHhWn4rSxNQ",
        "6vLVrWm0o/NiHp08Wx0K4LzWB/brKt1O6yNDK6urTJpMgiXyGQNZ7/WAWfCyTBOfnrMeTL44vxjj/347IRQ3/27lip7Nr+B4Mp4q",
        "IlivF1iGgvy34mWmklNs+xtHoQdEULxiwnUQOHpwihsxH/WP8rXgzAuVWzmPrMyzHAsROAajmvVEnL9VTo8aXIfIcTuN2RucWvfz",
        "bmQXiFEa/+3jr1ri3gEGyS+g/HD22pf49Fif5TiWto9uxxfv57KxfdLnFZO5/Px5ykHcbl2QBUZRWiESibSqlwBuauCKCG/4OArX",
        "tzRq7NR+MGARBqoGCns8pwN1KzS+PJPl9O4dyx5V0Uy0wodjVnHbKskZC5gNnXwebWOt4rjYjB/7KKwRyGQlJIKO6YyFxnaVIEJ6",
        "6C0bff/aXh3T+Hv3O+bDPo9BPF4hs7Be0WGlHBM8jFBV0yLNCH3tOVfpHuDeZCokPBlmJP27/192lzdwhYmjI+UzPHDi0Dp4XvX/",
        "Vj8wggE/BgkqhkiG9w0BBwGgggEwBIIBLDCCASgwggEkBgsqhkiG9w0BDAoBAqCBtDCBsTAcBgoqhkiG9w0BDAEDMA4ECCH7jNLi",
        "tNQGAgIIAASBkEHL7WA6lNL1T73J5zTxMK8JOZXSYO0xf0sfeaKpMUjQlwrAc5CKVF7t+/AUQLiIki+sfaTixk3qEWCf+k87GAt6",
        "2C1aEuETb4yekXlPtBGxK/5kJFK7+73WtLAeJVwxj/xr8T+q6TMYo9V+V/7y9v7jN587gUgXxwqSdlq2W7TG45GxspQphdiKZ44Q",
        "f/a10DFeMCMGCSqGSIb3DQEJFTEWBBQK65zzEgstj0LScgniPah2JL7NQjA3BgkqhkiG9w0BCRQxKh4oAHcAZQBiAHQAaQBnAGUA",
        "cgAtAHQAZQBzAHQALQBzAGUAYwBvAG4AZDAxMCEwCQYFKw4DAhoFAAQUklHJjC6noZX6lpwYoPWOBfqekmAECKUvdPDh1bBvAgII",
        "AA==",
    ].joined()
}
