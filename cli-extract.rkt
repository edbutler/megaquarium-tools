#lang errortrace racket

(provide extract)

(require
  "data.rkt"
  "core.rkt"
  "display.rkt")

(define (do-extract input-filename output-filename)
  (define argv (current-command-line-arguments))
  (define data (read-save (read-game-data) input-filename))
  (define do-write (thunk (write-tank-yaml data (current-output-port))))
  (cond
   [output-filename (with-output-to-file output-filename do-write #:exists 'replace)]
   [else (do-write)]))

(define (extract program-name args)
  (command-line
    #:program program-name 
    #:argv args
    #:usage-help
        "Extracts data from a save file into an editable yaml file, for use with other commands."
        "<input-file> is relative to the game's save directory. The extension is optional"
        "(e.g., all of 'mysave', 'mysave.sav', or /absolute/path/to/mysave will work)."
        "<output-file> is relative to the current directory."
        "If no <output-file> is supplied, will output to standard out."
    #:args (input-file [output-file #f])
    (do-extract input-file output-file)))