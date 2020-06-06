#lang racket

(provide
  default-l10n
  all-species
  species-ref
  read-save-file)

(require
  "core.rkt"
  "localization.rkt"
  "serialize.rkt")

(define language "en")

(define possible-save-directories (list (build-path (find-system-path 'home-dir) "Documents/My Games/Megaquarium/Saves")))
(define possible-data-directories (list "C:/Program Files (x86)/Steam" "D:/steam/"))
(define local-data-path "steamapps/common/Megaquarium/Megaquarium_Data/GameData/")
(define animal-path "Data/animals.data")
(define coral-path "Data/corals.data")
(define localization-path-format "Strings/~a/~a.json")
(define localization-files '("animals" "fishFood"))

(define extra-localization
  (hash 'fish "Fish"
        'clam "Clams"
        'gorgonian "Gorgonians"
        'starfish "Starfish"))

(define data-dir (findf directory-exists? (map (λ (d) (build-path d local-data-path)) possible-data-directories)))
(unless data-dir (error "Cannot find Megaquarium install directory!"))

(define default-l10n
  (let* ([map-filename (λ (f) (build-path data-dir (format localization-path-format language f)))]
         [files (map (compose1 read-json-file map-filename) localization-files)])
    (make-localization (cons extra-localization files))))

(define all-species
  (read-species-from-file
    #:animals (build-path data-dir animal-path)
    #:corals (build-path data-dir coral-path)))

(define (species-ref id)
  (findf (λ (a) (equal? id (species-id a))) all-species))

(define (read-save-file name)
  (set! name (path-replace-extension name ".sav"))
  (define filename
    (if (absolute-path? name)
      name
      (build-path (first possible-save-directories) name)))
  (read-save-from-file filename #:species all-species))

