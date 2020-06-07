#lang racket

(provide (all-defined-out))

(define language "en")

(define possible-save-directories
  (list (build-path (find-system-path 'home-dir) "Documents/My Games/Megaquarium/Saves")))
(define possible-data-directories
  (list "C:/Program Files (x86)/Steam" "D:/steam/"))
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

(define (find-data-dir)
  (or (findf directory-exists? (map (λ (d) (build-path d local-data-path)) possible-data-directories))
      (error "Cannot find Megaquarium install directory!")))

(define (save-file-path name)
  (set! name (path-replace-extension name ".sav"))
  (if (absolute-path? name)
    name
    (build-path (first possible-save-directories) name)))
